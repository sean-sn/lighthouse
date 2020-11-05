use slog::Logger;
use sloggers::Build;
use std::collections::HashSet;
use std::iter::FromIterator;
use types::{
    AggregateSignature, AttestationData, AttesterSlashing, Checkpoint, Epoch, Hash256,
    IndexedAttestation, MainnetEthSpec, Slot,
};

pub type E = MainnetEthSpec;

pub fn logger() -> Logger {
    if cfg!(feature = "test_logger") {
        sloggers::terminal::TerminalLoggerBuilder::new()
            .level(sloggers::types::Severity::Trace)
            .build()
            .unwrap()
    } else {
        sloggers::null::NullLoggerBuilder.build().unwrap()
    }
}

pub fn indexed_att(
    attesting_indices: impl AsRef<[u64]>,
    source_epoch: u64,
    target_epoch: u64,
    target_root: u64,
) -> IndexedAttestation<E> {
    IndexedAttestation {
        attesting_indices: attesting_indices.as_ref().to_vec().into(),
        data: AttestationData {
            slot: Slot::new(0),
            index: 0,
            beacon_block_root: Hash256::zero(),
            source: Checkpoint {
                epoch: Epoch::new(source_epoch),
                root: Hash256::from_low_u64_be(0),
            },
            target: Checkpoint {
                epoch: Epoch::new(target_epoch),
                root: Hash256::from_low_u64_be(target_root),
            },
        },
        signature: AggregateSignature::empty(),
    }
}

pub fn att_slashing(
    attestation_1: &IndexedAttestation<E>,
    attestation_2: &IndexedAttestation<E>,
) -> AttesterSlashing<E> {
    AttesterSlashing {
        attestation_1: attestation_1.clone(),
        attestation_2: attestation_2.clone(),
    }
}

pub fn hashset_intersection(
    attestation_1_indices: &[u64],
    attestation_2_indices: &[u64],
) -> HashSet<u64> {
    &HashSet::from_iter(attestation_1_indices.iter().copied())
        & &HashSet::from_iter(attestation_2_indices.iter().copied())
}

pub fn slashed_validators_from_slashings(slashings: &HashSet<AttesterSlashing<E>>) -> HashSet<u64> {
    slashings
        .iter()
        .flat_map(|slashing| {
            let att1 = &slashing.attestation_1;
            let att2 = &slashing.attestation_2;
            assert!(
                att1.is_double_vote(att2) || att1.is_surround_vote(att2),
                "invalid slashing: {:#?}",
                slashing
            );
            hashset_intersection(&att1.attesting_indices, &att2.attesting_indices)
        })
        .collect()
}

pub fn slashed_validators_from_attestations(
    attestations: &[IndexedAttestation<E>],
) -> HashSet<u64> {
    let mut slashed_validators = HashSet::new();
    // O(n^2) code, watch out.
    for att1 in attestations {
        for att2 in attestations {
            if att1 == att2 {
                continue;
            }

            if att1.is_double_vote(att2) || att1.is_surround_vote(att2) {
                slashed_validators.extend(hashset_intersection(
                    &att1.attesting_indices,
                    &att2.attesting_indices,
                ));
            }
        }
    }
    slashed_validators
}
