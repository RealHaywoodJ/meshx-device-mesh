// MeshX - The Immutable Global Device Mesh
// Proof of Physical Presence (PoP²) Implementation
// Copyright (c) 2025 MeshX Foundation

use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// TEE attestation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeeType {
    IntelSgx,
    ArmTrustZone,
    AppleSecureEnclave,
    AmdSev,
}

// Continental shards in MeshX network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shard {
    NorthAmerica,
    Europe,
    Asia,
    SouthAmerica,
    Africa,
    Oceania,
    Antarctica,
}

// Geographic coordinates for location verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: f32,
}

// TEE attestation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeAttestation {
    pub tee_type: TeeType,
    pub enclave_hash: [u8; 32],
    pub signer_pubkey: PublicKey,
    pub timestamp: u64,
    pub quote: Vec<u8>,
}

// Network latency measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    pub from_node: PublicKey,
    pub to_node: PublicKey,
    pub latency_ms: u32,
    pub timestamp: u64,
}

// VRF proof for random selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrfProof {
    pub input: [u8; 32],
    pub output: [u8; 32],
    pub proof: Vec<u8>,
}

// MeshX node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshXNode {
    pub pubkey: PublicKey,
    pub tee_attestation: TeeAttestation,
    pub geo_location: GeoLocation,
    pub shard: Shard,
    pub stake_amount: u64, // MESHX tokens staked
    pub reputation_score: f32,
    pub resources: NodeResources,
}

// Node computational resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu_cores: u32,
    pub ram_gb: u32,
    pub storage_gb: u64,
    pub bandwidth_mbps: u32,
    pub gpu_memory_gb: Option<u32>,
}

// Proof of Physical Presence validator
pub struct PopValidator {
    pub current_epoch: u64,
    pub nodes: HashMap<PublicKey, MeshXNode>,
    pub latency_matrix: HashMap<(PublicKey, PublicKey), u32>,
    pub minimum_nodes: usize,
}

impl PopValidator {
    pub fn new(minimum_nodes: usize) -> Self {
        Self {
            current_epoch: 0,
            nodes: HashMap::new(),
            latency_matrix: HashMap::new(),
            minimum_nodes,
        }
    }

    // Validate a node's proof of physical presence
    pub fn validate_node(&self, node: &MeshXNode) -> Result<bool, ValidationError> {
        // Step 1: Verify TEE attestation
        self.verify_tee_attestation(&node.tee_attestation)?;

        // Step 2: Verify geographic location via latency
        self.verify_location(&node.pubkey, &node.geo_location)?;

        // Step 3: Check minimum stake requirement
        if node.stake_amount < self.get_minimum_stake(&node.shard) {
            return Err(ValidationError::InsufficientStake);
        }

        // Step 4: Verify node resources
        self.verify_resources(&node.resources)?;

        Ok(true)
    }

    // Verify TEE attestation is valid and recent
    fn verify_tee_attestation(&self, attestation: &TeeAttestation) -> Result<(), ValidationError> {
        // Check attestation age (must be < 1 hour)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time - attestation.timestamp > 3600 {
            return Err(ValidationError::StaleAttestation);
        }

        // Verify enclave hash matches expected MeshX code
        let expected_hash = self.get_expected_enclave_hash();
        if attestation.enclave_hash != expected_hash {
            return Err(ValidationError::InvalidEnclaveCode);
        }

        // TODO: Implement actual TEE quote verification for each type
        match attestation.tee_type {
            TeeType::IntelSgx => self.verify_sgx_quote(&attestation.quote)?,
            TeeType::ArmTrustZone => self.verify_trustzone_quote(&attestation.quote)?,
            TeeType::AppleSecureEnclave => self.verify_secure_enclave_quote(&attestation.quote)?,
            TeeType::AmdSev => self.verify_sev_quote(&attestation.quote)?,
        }

        Ok(())
    }

    // Verify node's claimed location using latency triangulation
    fn verify_location(
        &self,
        node_pubkey: &PublicKey,
        claimed_location: &GeoLocation,
    ) -> Result<(), ValidationError> {
        // Get latency measurements to this node from others
        let measurements: Vec<_> = self
            .latency_matrix
            .iter()
            .filter(|((_, to), _)| to == node_pubkey)
            .map(|((from, _), latency)| (from, *latency))
            .collect();

        if measurements.len() < 3 {
            return Err(ValidationError::InsufficientLatencyData);
        }

        // Triangulate position based on latency
        let calculated_location = self.triangulate_position(&measurements)?;

        // Check if calculated location matches claimed (within accuracy)
        let distance = haversine_distance(&calculated_location, claimed_location);
        if distance > claimed_location.accuracy_meters as f64 {
            return Err(ValidationError::LocationMismatch);
        }

        Ok(())
    }

    // Calculate position from latency measurements
    fn triangulate_position(
        &self,
        measurements: &[(&PublicKey, u32)],
    ) -> Result<GeoLocation, ValidationError> {
        // Simplified triangulation based on speed of light
        // Real implementation would use more sophisticated algorithms

        let mut lat_sum = 0.0;
        let mut lon_sum = 0.0;
        let mut weight_sum = 0.0;

        for (peer_key, latency_ms) in measurements {
            if let Some(peer) = self.nodes.get(peer_key) {
                // Convert latency to approximate distance
                // Speed of light in fiber: ~200km/ms
                let distance_km = (*latency_ms as f64) * 200.0;

                // Weight by inverse distance
                let weight = 1.0 / distance_km;

                lat_sum += peer.geo_location.latitude * weight;
                lon_sum += peer.geo_location.longitude * weight;
                weight_sum += weight;
            }
        }

        Ok(GeoLocation {
            latitude: lat_sum / weight_sum,
            longitude: lon_sum / weight_sum,
            accuracy_meters: 50000.0, // 50km accuracy for now
        })
    }

    // Select validators for next epoch using VRF
    pub fn select_validators(&self, epoch: u64) -> Result<Vec<PublicKey>, ValidationError> {
        let mut selected = Vec::new();
        let mut candidates: Vec<_> = self.nodes.values().collect();

        // Sort by VRF output for deterministic selection
        candidates.sort_by_key(|node| {
            let vrf_input = self.compute_vrf_input(epoch, &node.pubkey);
            self.compute_vrf_output(&vrf_input, &node.pubkey)
        });

        // Select top N nodes weighted by stake
        for node in candidates.iter().take(self.get_validator_count()) {
            if self.validate_node(node).is_ok() {
                selected.push(node.pubkey);
            }
        }

        if selected.len() < self.minimum_nodes {
            return Err(ValidationError::InsufficientValidators);
        }

        Ok(selected)
    }

    // Compute VRF input for deterministic randomness
    fn compute_vrf_input(&self, epoch: u64, pubkey: &PublicKey) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(b"MESHX_VRF_INPUT");
        hasher.update(epoch.to_le_bytes());
        hasher.update(pubkey.as_bytes());
        
        let mut output = [0u8; 32];
        output.copy_from_slice(&hasher.finalize());
        output
    }

    // Compute VRF output (simplified - real implementation needs VRF)
    fn compute_vrf_output(&self, input: &[u8; 32], pubkey: &PublicKey) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(b"MESHX_VRF_OUTPUT");
        hasher.update(input);
        hasher.update(pubkey.as_bytes());
        
        let mut output = [0u8; 32];
        output.copy_from_slice(&hasher.finalize());
        output
    }

    // Assign node to continental shard based on location
    pub fn assign_shard(location: &GeoLocation) -> Shard {
        match (location.latitude, location.longitude) {
            (lat, lon) if lat > 15.0 && lat < 75.0 && lon > -170.0 && lon < -50.0 => {
                Shard::NorthAmerica
            }
            (lat, lon) if lat > 35.0 && lat < 75.0 && lon > -15.0 && lon < 40.0 => Shard::Europe,
            (lat, lon) if lat > -10.0 && lat < 55.0 && lon > 40.0 && lon < 150.0 => Shard::Asia,
            (lat, lon) if lat > -60.0 && lat < 15.0 && lon > -85.0 && lon < -30.0 => {
                Shard::SouthAmerica
            }
            (lat, lon) if lat > -40.0 && lat < 40.0 && lon > -20.0 && lon < 55.0 => Shard::Africa,
            (lat, lon) if lat > -50.0 && lat < -10.0 && lon > 110.0 && lon < 180.0 => {
                Shard::Oceania
            }
            (lat, _) if lat < -60.0 => Shard::Antarctica,
            _ => Shard::NorthAmerica, // Default fallback
        }
    }

    // Get minimum stake for a shard
    fn get_minimum_stake(&self, shard: &Shard) -> u64 {
        match shard {
            Shard::NorthAmerica | Shard::Europe | Shard::Asia => 100_000, // 100K MESHX
            Shard::SouthAmerica | Shard::Africa | Shard::Oceania => 50_000, // 50K MESHX
            Shard::Antarctica => 10_000, // 10K MESHX (encourage Antarctic nodes!)
        }
    }

    // Get validator count per shard
    fn get_validator_count(&self) -> usize {
        1000 // 1000 validators per shard
    }

    // Verify node has minimum resources
    fn verify_resources(&self, resources: &NodeResources) -> Result<(), ValidationError> {
        if resources.cpu_cores < 2 {
            return Err(ValidationError::InsufficientCPU);
        }
        if resources.ram_gb < 4 {
            return Err(ValidationError::InsufficientRAM);
        }
        if resources.storage_gb < 100 {
            return Err(ValidationError::InsufficientStorage);
        }
        if resources.bandwidth_mbps < 10 {
            return Err(ValidationError::InsufficientBandwidth);
        }

        Ok(())
    }

    // Placeholder TEE verification methods
    fn verify_sgx_quote(&self, quote: &[u8]) -> Result<(), ValidationError> {
        // TODO: Implement Intel SGX quote verification
        // This would involve EPID/DCAP verification
        if quote.is_empty() {
            return Err(ValidationError::InvalidQuote);
        }
        Ok(())
    }

    fn verify_trustzone_quote(&self, quote: &[u8]) -> Result<(), ValidationError> {
        // TODO: Implement ARM TrustZone attestation verification
        if quote.is_empty() {
            return Err(ValidationError::InvalidQuote);
        }
        Ok(())
    }

    fn verify_secure_enclave_quote(&self, quote: &[u8]) -> Result<(), ValidationError> {
        // TODO: Implement Apple Secure Enclave verification
        if quote.is_empty() {
            return Err(ValidationError::InvalidQuote);
        }
        Ok(())
    }

    fn verify_sev_quote(&self, quote: &[u8]) -> Result<(), ValidationError> {
        // TODO: Implement AMD SEV attestation verification
        if quote.is_empty() {
            return Err(ValidationError::InvalidQuote);
        }
        Ok(())
    }

    fn get_expected_enclave_hash(&self) -> [u8; 32] {
        // Hash of the expected MeshX validator code
        [0x42; 32] // Placeholder
    }
}

// Calculate distance between two geographic points (Haversine formula)
fn haversine_distance(loc1: &GeoLocation, loc2: &GeoLocation) -> f64 {
    const EARTH_RADIUS_M: f64 = 6_371_000.0;

    let lat1_rad = loc1.latitude.to_radians();
    let lat2_rad = loc2.latitude.to_radians();
    let delta_lat = (loc2.latitude - loc1.latitude).to_radians();
    let delta_lon = (loc2.longitude - loc1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_M * c
}

// Validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("TEE attestation is too old")]
    StaleAttestation,
    #[error("Invalid enclave code hash")]
    InvalidEnclaveCode,
    #[error("Invalid TEE quote")]
    InvalidQuote,
    #[error("Insufficient stake amount")]
    InsufficientStake,
    #[error("Not enough latency measurements")]
    InsufficientLatencyData,
    #[error("Location doesn't match latency triangulation")]
    LocationMismatch,
    #[error("Not enough validators available")]
    InsufficientValidators,
    #[error("Insufficient CPU cores")]
    InsufficientCPU,
    #[error("Insufficient RAM")]
    InsufficientRAM,
    #[error("Insufficient storage")]
    InsufficientStorage,
    #[error("Insufficient bandwidth")]
    InsufficientBandwidth,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_assignment() {
        // Test New York
        let ny = GeoLocation {
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy_meters: 1000.0,
        };
        assert_eq!(PopValidator::assign_shard(&ny), Shard::NorthAmerica);

        // Test London
        let london = GeoLocation {
            latitude: 51.5074,
            longitude: -0.1278,
            accuracy_meters: 1000.0,
        };
        assert_eq!(PopValidator::assign_shard(&london), Shard::Europe);

        // Test Tokyo
        let tokyo = GeoLocation {
            latitude: 35.6762,
            longitude: 139.6503,
            accuracy_meters: 1000.0,
        };
        assert_eq!(PopValidator::assign_shard(&tokyo), Shard::Asia);
    }

    #[test]
    fn test_haversine_distance() {
        let loc1 = GeoLocation {
            latitude: 40.7128,
            longitude: -74.0060,
            accuracy_meters: 1000.0,
        };

        let loc2 = GeoLocation {
            latitude: 51.5074,
            longitude: -0.1278,
            accuracy_meters: 1000.0,
        };

        let distance = haversine_distance(&loc1, &loc2);
        assert!((distance - 5_570_000.0).abs() < 10_000.0); // ~5570km ± 10km
    }
}