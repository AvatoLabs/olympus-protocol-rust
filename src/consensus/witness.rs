//! Witness management

use crate::{Address, Result, OlympusError};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Witness manager
pub struct WitnessManager {
    /// Current witnesses
    pub witnesses: Vec<Address>,
    /// Minimum witnesses required
    pub min_witnesses: u64,
    /// Maximum witnesses allowed
    pub max_witnesses: u64,
    /// Witness stakes (for PoS-like selection)
    pub stakes: HashMap<Address, u64>,
    /// Witness performance scores
    pub performance_scores: HashMap<Address, f64>,
}

/// Witness selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessCriteria {
    /// Minimum stake required
    pub min_stake: u64,
    /// Minimum performance score
    pub min_performance: f64,
    /// Maximum age of witness
    pub max_age: u64,
}

impl WitnessManager {
    /// Create new witness manager
    pub fn new(min_witnesses: u64, max_witnesses: u64) -> Self {
        Self {
            witnesses: Vec::new(),
            min_witnesses,
            max_witnesses,
            stakes: HashMap::new(),
            performance_scores: HashMap::new(),
        }
    }

    /// Add witness
    pub fn add_witness(&mut self, witness: Address) -> Result<()> {
        if self.witnesses.len() >= self.max_witnesses as usize {
            return Err(OlympusError::Consensus("Maximum witnesses reached".to_string()));
        }
        
        if !self.witnesses.contains(&witness) {
            self.witnesses.push(witness);
        }
        
        Ok(())
    }

    /// Remove witness
    pub fn remove_witness(&mut self, witness: Address) -> Result<()> {
        if let Some(pos) = self.witnesses.iter().position(|&w| w == witness) {
            self.witnesses.remove(pos);
            Ok(())
        } else {
            Err(OlympusError::Consensus("Witness not found".to_string()))
        }
    }

    /// Check if we have enough witnesses
    pub fn has_enough_witnesses(&self) -> bool {
        self.witnesses.len() >= self.min_witnesses as usize
    }

    /// Set witness stake
    pub fn set_stake(&mut self, witness: Address, stake: u64) {
        self.stakes.insert(witness, stake);
    }

    /// Get witness stake
    pub fn get_stake(&self, witness: Address) -> u64 {
        self.stakes.get(&witness).cloned().unwrap_or(0)
    }

    /// Update witness performance score
    pub fn update_performance(&mut self, witness: Address, score: f64) {
        self.performance_scores.insert(witness, score);
    }

    /// Get witness performance score
    pub fn get_performance(&self, witness: Address) -> f64 {
        self.performance_scores.get(&witness).cloned().unwrap_or(0.0)
    }

    /// Select witnesses based on criteria
    pub fn select_witnesses(&self, criteria: &WitnessCriteria) -> Vec<Address> {
        let mut candidates: Vec<_> = self.witnesses.iter()
            .filter(|&&witness| {
                let stake = self.get_stake(witness);
                let performance = self.get_performance(witness);
                
                stake >= criteria.min_stake && performance >= criteria.min_performance
            })
            .cloned()
            .collect();

        // Sort by stake and performance
        candidates.sort_by(|a, b| {
            let stake_a = self.get_stake(*a);
            let stake_b = self.get_stake(*b);
            let perf_a = self.get_performance(*a);
            let perf_b = self.get_performance(*b);
            
            // Primary sort by stake, secondary by performance
            match stake_b.cmp(&stake_a) {
                std::cmp::Ordering::Equal => perf_b.partial_cmp(&perf_a).unwrap_or(std::cmp::Ordering::Equal),
                other => other,
            }
        });

        // Take up to max_witnesses
        candidates.into_iter()
            .take(self.max_witnesses as usize)
            .collect()
    }

    /// Validate witness eligibility
    pub fn is_eligible(&self, witness: Address, criteria: &WitnessCriteria) -> bool {
        let stake = self.get_stake(witness);
        let performance = self.get_performance(witness);
        
        stake >= criteria.min_stake && performance >= criteria.min_performance
    }

    /// Get witness statistics
    pub fn get_statistics(&self) -> WitnessStatistics {
        let total_stake: u64 = self.stakes.values().sum();
        let avg_performance = if !self.performance_scores.is_empty() {
            self.performance_scores.values().sum::<f64>() / self.performance_scores.len() as f64
        } else {
            0.0
        };

        WitnessStatistics {
            total_witnesses: self.witnesses.len(),
            total_stake,
            average_performance: avg_performance,
            min_witnesses: self.min_witnesses,
            max_witnesses: self.max_witnesses,
        }
    }

    /// Rotate witnesses based on performance
    pub fn rotate_witnesses(&mut self, new_witnesses: Vec<Address>) -> Result<()> {
        if new_witnesses.len() < self.min_witnesses as usize {
            return Err(OlympusError::Consensus("Not enough witnesses for rotation".to_string()));
        }

        if new_witnesses.len() > self.max_witnesses as usize {
            return Err(OlympusError::Consensus("Too many witnesses for rotation".to_string()));
        }

        self.witnesses = new_witnesses;
        Ok(())
    }
}

/// Witness statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessStatistics {
    /// Total number of witnesses
    pub total_witnesses: usize,
    /// Total stake of all witnesses
    pub total_stake: u64,
    /// Average performance score
    pub average_performance: f64,
    /// Minimum witnesses required
    pub min_witnesses: u64,
    /// Maximum witnesses allowed
    pub max_witnesses: u64,
}

impl Default for WitnessManager {
    fn default() -> Self {
        Self::new(3, 21)
    }
}

impl Default for WitnessCriteria {
    fn default() -> Self {
        Self {
            min_stake: 1000,
            min_performance: 0.5,
            max_age: 1000,
        }
    }
}

impl WitnessCriteria {
    /// Create new witness criteria with configurable parameters
    pub fn new(min_stake: u64, min_performance: f64, max_age: u64) -> Self {
        Self {
            min_stake,
            min_performance,
            max_age,
        }
    }
}
