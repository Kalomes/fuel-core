use crate::database::{
    storage::DatabaseColumn,
    Column,
    Database,
    Result as DatabaseResult,
};
use fuel_core_chain_config::{
    ContractBalanceConfig,
    ContractConfig,
    ContractStateConfig,
};
use fuel_core_storage::{
    iter::IterDirection,
    not_found,
    tables::{
        ContractsInfo,
        ContractsLatestUtxo,
        ContractsRawCode,
    },
    ContractsAssetKey,
    Error as StorageError,
    Mappable,
    Result as StorageResult,
    StorageAsRef,
    StorageInspect,
    StorageMutate,
    StorageRead,
    StorageSize,
    StorageWrite,
};
use fuel_core_types::{
    entities::contract::ContractUtxoInfo,
    fuel_tx::Contract,
    fuel_types::{
        AssetId,
        Bytes32,
        ContractId,
        Word,
    },
};
use std::borrow::Cow;

impl DatabaseColumn for ContractsLatestUtxo {
    fn column() -> Column {
        Column::ContractsLatestUtxo
    }
}

impl StorageInspect<ContractsRawCode> for Database {
    type Error = StorageError;

    fn get(
        &self,
        key: &<ContractsRawCode as Mappable>::Key,
    ) -> Result<Option<Cow<<ContractsRawCode as Mappable>::OwnedValue>>, Self::Error>
    {
        Ok(self
            .read_alloc(key.as_ref(), Column::ContractsRawCode)?
            .map(|v| Cow::Owned(Contract::from(v))))
    }

    fn contains_key(
        &self,
        key: &<ContractsRawCode as Mappable>::Key,
    ) -> Result<bool, Self::Error> {
        self.contains_key(key.as_ref(), Column::ContractsRawCode)
            .map_err(Into::into)
    }
}

// # Dev-note: The value of the `ContractsRawCode` has a unique implementation of serialization
// and deserialization. Because the value is a contract byte code represented by bytes,
// we don't use `serde::Deserialization` and `serde::Serialization` for `Vec`, because we don't
// need to store the size of the contract. We store/load raw bytes.
impl StorageMutate<ContractsRawCode> for Database {
    fn insert(
        &mut self,
        key: &<ContractsRawCode as Mappable>::Key,
        value: &<ContractsRawCode as Mappable>::Value,
    ) -> Result<Option<<ContractsRawCode as Mappable>::OwnedValue>, Self::Error> {
        let existing =
            Database::replace(self, key.as_ref(), Column::ContractsRawCode, value)?;
        Ok(existing.1.map(Contract::from))
    }

    fn remove(
        &mut self,
        key: &<ContractsRawCode as Mappable>::Key,
    ) -> Result<Option<<ContractsRawCode as Mappable>::OwnedValue>, Self::Error> {
        Ok(
            <Self as StorageWrite<ContractsRawCode>>::take(self, key)?
                .map(Contract::from),
        )
    }
}

impl StorageSize<ContractsRawCode> for Database {
    fn size_of_value(&self, key: &ContractId) -> Result<Option<usize>, Self::Error> {
        Ok(self.size_of_value(key.as_ref(), Column::ContractsRawCode)?)
    }
}

impl StorageRead<ContractsRawCode> for Database {
    fn read(
        &self,
        key: &ContractId,
        buf: &mut [u8],
    ) -> Result<Option<usize>, Self::Error> {
        Ok(self.read(key.as_ref(), Column::ContractsRawCode, buf)?)
    }

    fn read_alloc(&self, key: &ContractId) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.read_alloc(key.as_ref(), Column::ContractsRawCode)?)
    }
}

impl StorageWrite<ContractsRawCode> for Database {
    fn write(&mut self, key: &ContractId, buf: Vec<u8>) -> Result<usize, Self::Error> {
        Ok(Database::write(
            self,
            key.as_ref(),
            Column::ContractsRawCode,
            &buf,
        )?)
    }

    fn replace(
        &mut self,
        key: &<ContractsRawCode as Mappable>::Key,
        buf: Vec<u8>,
    ) -> Result<(usize, Option<Vec<u8>>), <Self as StorageInspect<ContractsRawCode>>::Error>
    where
        Self: StorageSize<ContractsRawCode>,
    {
        Ok(Database::replace(
            self,
            key.as_ref(),
            Column::ContractsRawCode,
            &buf,
        )?)
    }

    fn take(
        &mut self,
        key: &<ContractsRawCode as Mappable>::Key,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(Database::take(
            self,
            key.as_ref(),
            Column::ContractsRawCode,
        )?)
    }
}

impl Database {
    pub fn iter_contract_state_configs(
        &self,
    ) -> impl Iterator<Item = StorageResult<ContractStateConfig>> + '_ {
        self.iter_all::<Vec<u8>, Bytes32>(Column::ContractsState, None)
            .map(|res| {
                let res = res?;
                let contract_id = Bytes32::new(res.0[0..32].try_into()?);
                let key = Bytes32::new(res.0[32..].try_into()?);
                let value = res.1;

                Ok(ContractStateConfig {
                    contract_id,
                    key,
                    value,
                })
            })
    }

    pub fn iter_contract_balance_configs(
        &self,
    ) -> impl Iterator<Item = StorageResult<ContractBalanceConfig>> + '_ {
        self.iter_all::<Vec<u8>, u64>(Column::ContractsAssets, None)
            .map(|res| {
                let res = res?;

                let contract_id = Bytes32::new(res.0[0..32].try_into()?);
                let asset_id = AssetId::new(res.0[32..].try_into()?);
                let amount = res.1;

                Ok(ContractBalanceConfig {
                    contract_id,
                    asset_id,
                    amount,
                })
            })
    }

    pub fn get_contract_config(
        &self,
        contract_id: ContractId,
    ) -> StorageResult<ContractConfig> {
        let code = self
            .storage::<ContractsRawCode>()
            .get(&contract_id)?
            .map(|v| {
                let code: Vec<u8> = v.into_owned().into();
                code
            })
            .ok_or_else(|| not_found!("ContractsRawCode"))?;

        let (salt, _) = self
            .storage::<ContractsInfo>()
            .get(&contract_id)
            .unwrap()
            .expect("Contract does not exist")
            .into_owned();

        let ContractUtxoInfo {
            utxo_id,
            tx_pointer,
        } = self
            .storage::<ContractsLatestUtxo>()
            .get(&contract_id)
            .unwrap()
            .expect("contract does not exist")
            .into_owned();

        Ok(ContractConfig {
            contract_id,
            code,
            salt,
            tx_id: Some(*utxo_id.tx_id()),
            output_index: Some(utxo_id.output_index()),
            tx_pointer_block_height: Some(tx_pointer.block_height()),
            tx_pointer_tx_idx: Some(tx_pointer.tx_index()),
        })
    }

    pub fn contract_balances(
        &self,
        contract: ContractId,
        start_asset: Option<AssetId>,
        direction: Option<IterDirection>,
    ) -> impl Iterator<Item = DatabaseResult<(AssetId, Word)>> + '_ {
        self.iter_all_filtered::<Vec<u8>, Word, _, _>(
            Column::ContractsAssets,
            Some(contract),
            start_asset.map(|asset_id| ContractsAssetKey::new(&contract, &asset_id)),
            direction,
        )
        .map(|res| {
            res.map(|(key, balance)| {
                (AssetId::new(key[32..].try_into().unwrap()), balance)
            })
        })
    }

    pub fn contract_states(
        &self,
        contract_id: ContractId,
    ) -> impl Iterator<Item = DatabaseResult<(Bytes32, Bytes32)>> + '_ {
        self.iter_all_by_prefix::<Vec<u8>, Bytes32, _>(
            Column::ContractsState,
            Some(contract_id),
        )
        .map(|res| -> DatabaseResult<_> {
            let safe_res = res?;

            // We don't need to store ContractId which is the first 32 bytes of this
            // key, as this Vec is already attached to that ContractId
            let state_key = Bytes32::new(safe_res.0[32..].try_into()?);
            let state_value = safe_res.1;

            Ok((state_key, state_value))
        })
    }

    pub fn iter_contract_configs(
        &self,
    ) -> impl Iterator<Item = StorageResult<ContractConfig>> + '_ {
        self.iter_all::<Vec<u8>, Word>(Column::ContractsRawCode, None)
            .map(|raw_contract_id| {
                let raw_contract_id = raw_contract_id?;
                let contract_id = ContractId::new(raw_contract_id.0[..32].try_into()?);
                self.get_contract_config(contract_id)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_core_storage::StorageAsMut;
    use fuel_core_types::fuel_tx::{
        Contract,
        TxId,
        TxPointer,
        UtxoId,
    };
    use rand::{
        RngCore,
        SeedableRng,
    };

    #[test]
    fn raw_code_get() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let contract: Contract = Contract::from(vec![32u8]);

        let database = &mut Database::default();

        database
            .storage::<ContractsRawCode>()
            .insert(&contract_id, contract.as_ref())
            .unwrap();

        assert_eq!(
            database
                .storage::<ContractsRawCode>()
                .get(&contract_id)
                .unwrap()
                .unwrap()
                .into_owned(),
            contract
        );
    }

    #[test]
    fn raw_code_put() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let contract: Contract = Contract::from(vec![32u8]);

        let database = &mut Database::default();
        database
            .storage::<ContractsRawCode>()
            .insert(&contract_id, contract.as_ref())
            .unwrap();

        let returned: Contract = database
            .storage::<ContractsRawCode>()
            .get(&contract_id)
            .unwrap()
            .unwrap()
            .into_owned();
        assert_eq!(returned, contract);
    }

    #[test]
    fn raw_code_put_huge_contract() {
        let rng = &mut rand::rngs::StdRng::seed_from_u64(2322u64);
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let mut bytes = vec![0; 16 * 1024 * 1024];
        rng.fill_bytes(bytes.as_mut());
        let contract: Contract = Contract::from(bytes);

        let database = &mut Database::default();
        database
            .storage::<ContractsRawCode>()
            .insert(&contract_id, contract.as_ref())
            .unwrap();

        let returned: Contract = database
            .storage::<ContractsRawCode>()
            .get(&contract_id)
            .unwrap()
            .unwrap()
            .into_owned();
        assert_eq!(returned, contract);
    }

    #[test]
    fn raw_code_remove() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let contract: Contract = Contract::from(vec![32u8]);

        let database = &mut Database::default();
        database
            .storage::<ContractsRawCode>()
            .insert(&contract_id, contract.as_ref())
            .unwrap();

        database
            .storage::<ContractsRawCode>()
            .remove(&contract_id)
            .unwrap();

        assert!(!database
            .storage::<ContractsRawCode>()
            .contains_key(&contract_id)
            .unwrap());
    }

    #[test]
    fn raw_code_exists() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let contract: Contract = Contract::from(vec![32u8]);

        let database = &mut Database::default();
        database
            .storage::<ContractsRawCode>()
            .insert(&contract_id, contract.as_ref())
            .unwrap();

        assert!(database
            .storage::<ContractsRawCode>()
            .contains_key(&contract_id)
            .unwrap());
    }

    #[test]
    fn latest_utxo_get() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let utxo_id: UtxoId = UtxoId::new(TxId::new([2u8; 32]), 4);
        let tx_pointer = TxPointer::new(1.into(), 5);
        let utxo_info = ContractUtxoInfo {
            utxo_id,
            tx_pointer,
        };
        let database = &mut Database::default();

        database
            .storage::<ContractsLatestUtxo>()
            .insert(&contract_id, &utxo_info)
            .unwrap();

        assert_eq!(
            database
                .storage::<ContractsLatestUtxo>()
                .get(&contract_id)
                .unwrap()
                .unwrap()
                .into_owned(),
            utxo_info
        );
    }

    #[test]
    fn latest_utxo_put() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let utxo_id: UtxoId = UtxoId::new(TxId::new([2u8; 32]), 4);
        let tx_pointer = TxPointer::new(1.into(), 5);
        let utxo_info = ContractUtxoInfo {
            utxo_id,
            tx_pointer,
        };

        let database = &mut Database::default();
        database
            .storage::<ContractsLatestUtxo>()
            .insert(&contract_id, &utxo_info)
            .unwrap();

        let returned: ContractUtxoInfo = database
            .storage::<ContractsLatestUtxo>()
            .get(&contract_id)
            .unwrap()
            .unwrap()
            .into_owned();
        assert_eq!(returned, utxo_info);
    }

    #[test]
    fn latest_utxo_remove() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let utxo_id: UtxoId = UtxoId::new(TxId::new([2u8; 32]), 4);
        let tx_pointer = TxPointer::new(1.into(), 5);

        let database = &mut Database::default();
        database
            .storage::<ContractsLatestUtxo>()
            .insert(
                &contract_id,
                &ContractUtxoInfo {
                    utxo_id,
                    tx_pointer,
                },
            )
            .unwrap();

        database
            .storage::<ContractsLatestUtxo>()
            .remove(&contract_id)
            .unwrap();

        assert!(!database
            .storage::<ContractsLatestUtxo>()
            .contains_key(&contract_id)
            .unwrap());
    }

    #[test]
    fn latest_utxo_exists() {
        let contract_id: ContractId = ContractId::from([1u8; 32]);
        let utxo_id: UtxoId = UtxoId::new(TxId::new([2u8; 32]), 4);
        let tx_pointer = TxPointer::new(1.into(), 5);

        let database = &mut Database::default();
        database
            .storage::<ContractsLatestUtxo>()
            .insert(
                &contract_id,
                &ContractUtxoInfo {
                    utxo_id,
                    tx_pointer,
                },
            )
            .unwrap();

        assert!(database
            .storage::<ContractsLatestUtxo>()
            .contains_key(&contract_id)
            .unwrap());
    }
}
