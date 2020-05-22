use std::convert::From;
use std::ops::Range;

use rand::{
    prelude::*,
    Rng,
    distributions::weighted::alias_method::WeightedIndex,
};
use indicatif::{ProgressBar, ProgressStyle, ProgressIterator};

use crate::graph::{Contact, Graph};


/// Edge-Markovian graph model properties
pub struct EdgeMarkovian {
    pub creation_probability: f32,
    pub deletion_probability: f32,
    pub duration: i32,
    pub number_of_nodes: i32,
}

/// Create a `Graph` from an Edge-Markovian model.
/// We consider here that at `t = 0`, there are no links
///
/// The complexity is `O(n^2 * T)` with `n` the number of nodes and `T` the total duration of the
/// experiment.
impl From<EdgeMarkovian> for Graph {
    fn from(model: EdgeMarkovian) -> Graph {
        // Represents a pair : (n1, n2, is connected, id of the contact assigned)
        let mut pairs: Vec<(i32, i32, bool, usize)> = Vec::with_capacity(
            ((model.number_of_nodes * (model.number_of_nodes - 1)) / 2) as usize,
        ); // nCr(n, 2) = n(n-1)/2

        for i_node1 in 1..=model.number_of_nodes {
            for i_node2 in (i_node1 + 1)..=model.number_of_nodes {
                pairs.push((i_node1, i_node2, false, core::usize::MAX));
            }
        }

        let mut rng = rand::thread_rng();
        let mut rand_num: f32;

        let mut contacts: Vec<Contact> = Vec::with_capacity(
            (model.creation_probability * model.duration as f32) as usize * pairs.len(),
        );

        // Progress bar
        let pb = ProgressBar::new(model.duration as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.yellow/blue}] {percent}% ({eta})")
            .progress_chars("#>-"));

        for t in (1..=model.duration).progress_with(pb) {
            for pair in pairs.iter_mut() {
                // Generate number in (0, 1[
                rand_num = rng.gen();

                // If (n_1, n_2) is in E_{t-1}, delete pair with probability d
                if pair.2 == true && rand_num <= model.deletion_probability {
                    contacts[pair.3].end = t;

                    pair.2 = false;
                    pair.3 = core::usize::MAX;
                }

                // Generate number in (0, 1[
                rand_num = rng.gen();

                // If (n_1, n_2) is not in E_{t-1}, create pair with probability p
                if pair.2 == false && rand_num <= model.creation_probability {
                    contacts.push(Contact {
                        start: t,
                        couple: (pair.0, pair.1),
                        end: 0,
                    });

                    pair.2 = true;
                    pair.3 = contacts.len() - 1;
                }
            }
        }

        // Remove the contacts that could not end
        contacts = contacts.into_iter().filter(|c| c.end != 0).collect();

        let graph = Graph {
            duration: model.duration,
            nodes: Range {
                start: 1,
                end: model.number_of_nodes,
            }
            .collect(),
            contacts,
        };

        graph
    }
}

/// Edge-Markovian model properties with time dependent creation/deletion probabilities
/// `creation_probability` and `deletion_probability` must have `n = duration` values
pub struct TimeDependentEdgeMarkovian {
    pub creation_probability: Vec<f32>,
    pub deletion_probability: Vec<f32>,
    pub duration: i32,
    pub number_of_nodes: i32,
}

/// Create a `Graph` from a Edge-Markovian model with time dependent creation and deletion
/// probabilities. We consider here that at `t = 0`, there are no links
impl From<TimeDependentEdgeMarkovian> for Graph {
    fn from(model: TimeDependentEdgeMarkovian) -> Graph {
        // Represents a pair : (n1, n2, is connected, id of the contact assigned)
        let mut pairs: Vec<(i32, i32, bool, usize)> = Vec::with_capacity(
            ((model.number_of_nodes * (model.number_of_nodes - 1)) / 2) as usize,
        ); // nCr(n, 2) = n(n-1)/2

        for i_node1 in 1..=model.number_of_nodes {
            for i_node2 in (i_node1 + 1)..=model.number_of_nodes {
                pairs.push((i_node1, i_node2, false, core::usize::MAX));
            }
        }

        let mut rng = rand::thread_rng();
        let mut rand_num: f32;

        let mut contacts: Vec<Contact> = Vec::with_capacity(
            (model.duration / 100) as usize * pairs.len(),
        );

        // Progress bar
        let pb = ProgressBar::new(model.duration as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.yellow/blue}] {percent}% ({eta})")
            .progress_chars("#>-"));

        for t in (1..=model.duration).progress_with(pb) {
            for pair in pairs.iter_mut() {
                // Generate number in (0, 1[
                rand_num = rng.gen();

                // If (n_1, n_2) is in E_{t-1}, delete pair with probability d
                if pair.2 == true && rand_num <= model.deletion_probability[t as usize] {
                    contacts[pair.3].end = t;

                    pair.2 = false;
                    pair.3 = core::usize::MAX;
                }

                // Generate number in (0, 1[
                rand_num = rng.gen();

                // If (n_1, n_2) is not in E_{t-1}, create pair with probability p
                if pair.2 == false && rand_num <= model.creation_probability[t as usize] {
                    contacts.push(Contact {
                        start: t,
                        couple: (pair.0, pair.1),
                        end: 0,
                    });

                    pair.2 = true;
                    pair.3 = contacts.len() - 1;
                }
            }
        }

        // Remove the contacts that could not end
        contacts = contacts.into_iter().filter(|c| c.end != 0).collect();

        let graph = Graph {
            duration: model.duration,
            nodes: Range {
                start: 1,
                end: model.number_of_nodes,
            }
                .collect(),
            contacts,
        };

        graph
    }
}

/// Edge-Markovian model properties with time dependent creation/deletion probabilities and delayed
/// nodes
pub struct DelayedTimeDependentEdgeMarkovian {
    pub creation_probability: Vec<f32>,
    pub deletion_probability: Vec<f32>,
    pub intercontacts_histogram: Vec<i32>,
    pub duration: i32,
    pub number_of_nodes: i32,
}

/// Create a `Graph` from a Edge-Markovian model with time dependent creation and deletion
/// probabilities and delayed nodes. We consider here that at `t = 0`, there are no links
impl From<DelayedTimeDependentEdgeMarkovian> for Graph {
    fn from(model: DelayedTimeDependentEdgeMarkovian) -> Graph {
        // Represents a pair : (n1, n2, is connected, id of the contact assigned, the time before
        // any new connexion is prohibited)
        let mut pairs: Vec<(i32, i32, bool, usize, i32)> = Vec::with_capacity(
            ((model.number_of_nodes * (model.number_of_nodes - 1)) / 2) as usize,
        ); // nCr(n, 2) = n(n-1)/2

        for i_node1 in 1..=model.number_of_nodes {
            for i_node2 in (i_node1 + 1)..=model.number_of_nodes {
                pairs.push((i_node1, i_node2, false, core::usize::MAX, core::i32::MAX));
            }
        }

        let mut rng = rand::thread_rng();
        let mut rand_num: f32;

        let mut contacts: Vec<Contact> = Vec::with_capacity(
            (model.duration / 100) as usize * pairs.len(),
        );

        // Create the inter-contacts distribution from the histogram
        let values: Vec<i32> = (-1..model.intercontacts_histogram.len() as i32 -1).collect();
        let intercontacts_dist = WeightedIndex::new(
            model.intercontacts_histogram
        ).unwrap();

        // Progress bar
        let pb = ProgressBar::new(model.duration as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.yellow/blue}] {percent}% ({eta})")
            .progress_chars("#>-"));

        for t in (1..=model.duration).progress_with(pb) {
            for pair in pairs.iter_mut() {
                // Generate number in (0, 1[
                rand_num = rng.gen();

                // If (n_1, n_2) is in E_{t-1}, delete pair with probability d
                if pair.2 == true && rand_num <= model.deletion_probability[t as usize] {
                    contacts[pair.3].end = t;

                    pair.2 = false;
                    pair.3 = core::usize::MAX;

                    let delay = values[intercontacts_dist.sample(&mut rng)];
                    pair.4 = t + delay;
                }

                // If (n_1, n_2) is not in E_{t-1}, create pair with probability p
                // and pair is not delayed
                // Generate number in (0, 1[
                rand_num = rng.gen();

                if pair.2 == false
                    && pair.4 >= t
                    && rand_num <= model.creation_probability[t as usize] {
                    contacts.push(Contact {
                        start: t,
                        couple: (pair.0, pair.1),
                        end: 0,
                    });

                    pair.2 = true;
                    pair.3 = contacts.len() - 1;
                    pair.4 = core::i32::MAX;

                    break;
                }
            }
        }

        // Remove the contacts that could not end
        contacts = contacts.into_iter().filter(|c| c.end != 0).collect();

        let graph = Graph {
            duration: model.duration,
            nodes: Range {
                start: 1,
                end: model.number_of_nodes,
            }
                .collect(),
            contacts,
        };

        graph
    }
}