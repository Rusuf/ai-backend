use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Rule<T: Eq + Hash> {
    pub lhs: Vec<T>,
    pub rhs: Vec<T>,
    pub confidence: f64,
}

pub fn apriori<T: Eq + Hash + Clone + Ord>(
    transactions: &[Vec<T>],
    min_support: f64,
    min_confidence: f64,
) -> Vec<Rule<T>> {
    let frequent_itemsets = get_frequent_itemsets(transactions, min_support);
    let mut rules = Vec::new();

    for (itemset, &itemset_support) in &frequent_itemsets {
        if itemset.len() > 1 {
            let subsets = (1..itemset.len())
                .flat_map(|k| combinations(itemset, k))
                .collect::<Vec<Vec<T>>>();

            for lhs in subsets {
                if let Some(&lhs_support) = frequent_itemsets.get(&lhs) {
                    let confidence = itemset_support / lhs_support;
                    if confidence >= min_confidence {
                        let rhs: Vec<T> = itemset
                            .iter()
                            .filter(|item| !lhs.contains(item))
                            .cloned()
                            .collect();
                        if !rhs.is_empty() {
                            rules.push(Rule {
                                lhs,
                                rhs,
                                confidence,
                            });
                        }
                    }
                }
            }
        }
    }
    rules
}

fn get_frequent_itemsets<T: Eq + Hash + Clone + Ord>(
    transactions: &[Vec<T>],
    min_support: f64,
) -> HashMap<Vec<T>, f64> {
    let mut item_counts: HashMap<T, usize> = HashMap::new();
    for transaction in transactions {
        for item in transaction {
            *item_counts.entry(item.clone()).or_insert(0) += 1;
        }
    }

    let num_transactions = transactions.len() as f64;
    let mut frequent_itemsets: HashMap<Vec<T>, f64> = HashMap::new();

    // Level 1 frequent itemsets
    let l1: HashSet<Vec<T>> = item_counts
        .into_iter()
        .filter_map(|(item, count)| {
            let support = count as f64 / num_transactions;
            if support >= min_support {
                let itemset = vec![item];
                frequent_itemsets.insert(itemset.clone(), support);
                Some(itemset)
            } else {
                None
            }
        })
        .collect();

    let mut lk = l1;
    let mut k = 2;

    while !lk.is_empty() {
        let ck = generate_candidates(&lk, k);
        let mut lk_next = HashSet::new();
        
        // Optimization: Pre-compute transaction sets for faster lookup
        let transaction_sets: Vec<HashSet<T>> = transactions
            .iter()
            .map(|t| t.iter().cloned().collect())
            .collect();

        for mut candidate in ck {
            let mut count = 0;
            let candidate_set: HashSet<_> = candidate.iter().cloned().collect();
            
            for t_set in &transaction_sets {
                if candidate_set.is_subset(t_set) {
                    count += 1;
                }
            }

            let support = count as f64 / num_transactions;
            if support >= min_support {
                candidate.sort();
                frequent_itemsets.insert(candidate.clone(), support);
                lk_next.insert(candidate);
            }
        }
        lk = lk_next;
        k += 1;
    }

    frequent_itemsets
}

fn generate_candidates<T: Eq + Hash + Clone + Ord>(
    prev_lk: &HashSet<Vec<T>>,
    k: usize,
) -> HashSet<Vec<T>> {
    let mut candidates = HashSet::new();
    let items: Vec<_> = prev_lk.iter().collect();
    for i in 0..items.len() {
        for j in (i + 1)..items.len() {
            // Join step: merge two itemsets if they share k-2 items
            // For k=2, they share 0 items (just merge)
            let set_i: HashSet<_> = items[i].iter().cloned().collect();
            let set_j: HashSet<_> = items[j].iter().cloned().collect();
            
            let union: Vec<_> = set_i.union(&set_j).cloned().collect();
            
            if union.len() == k {
                let mut sorted_union = union;
                sorted_union.sort();
                candidates.insert(sorted_union);
            }
        }
    }
    candidates
}

fn combinations<T: Clone + Ord>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.is_empty() {
        return vec![];
    }
    let mut result = combinations(&items[1..], k);
    for mut combo in combinations(&items[1..], k - 1) {
        combo.push(items[0].clone());
        combo.sort();
        result.push(combo);
    }
    result
}
