use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::Config;

const MAX_NGRAMS: u32 = 1_000;

const FILENAME_WEIGHT: u32 = 2;

const NGRAM_SIZE: u32 = 3;
const NGRAM_WEIGHT: u32 = 1;
const NGRAM_MIN_MATCHES: u32 = 3;
const NGRAMS_PER_THREAD: u32 = 1_000;

pub fn run_ngram_approach_v2(input: &String, config: &Config) {
    let print_verbose = if config.verbose {
        |s: &str| {
            println!("{}", s);
        }
    } else {
        |_s: &str| {}
    };

    let mut result: Vec<(&String, i32)> = vec![];
    let amount_matching_ngrams: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));

    print_verbose("Reading cache file");
    // TODO: Change this so cache path is passed through funciton instead.
    let cache_lines: Vec<String> = std::fs::read_to_string("./cache")
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();


    let input_ngrams = generate_ngram_bytes(NGRAM_SIZE, &input);

    print_verbose("Beginning ngram creation");
    // Split up cache file in to queue of vectors of strings (1000 in each?)
    // Spawn threads
    //
    let line_queue: Arc<Mutex<VecDeque<Vec<String>>>> = Arc::new(Mutex::new(VecDeque::new()));
    let mut tmp_counter = 0;
    let mut tmp_vec: Vec<String> = vec![];
    let mut line_counter: usize = 0;
    // TODO: Double check efficiency of this (the .iter() call etc.)
    // Also aquire lock for queue and then drop after length call pls
    loop {
        let line: String = cache_lines.get(line_counter).unwrap().clone();
        tmp_vec.push(line);
        tmp_counter += 1;
        if tmp_counter >= NGRAMS_PER_THREAD {
            line_queue.lock().unwrap().push_back(tmp_vec);
            tmp_vec = vec![];
            tmp_counter = 0;
        }
        line_counter += 1;
        if line_counter == cache_lines.len() {
            break; 
        }
    }
    line_queue.lock().unwrap().push_back(tmp_vec);
    
    print_verbose(format!("Lenght of line queue: {}", line_queue.lock().unwrap().len()).as_str());

    // loop {
    //     print_verbose(format!("Size of hashmap: {}", amount_matching_ngrams.len()).as_str());
    //     let mut tmp: Vec<&String> = vec![];
    //     loop {
    //         tmp.push(if let Some(val) = cache_lines.get(current_line as usize) {
    //             &val
    //         } else {
    //             current_line += 1;
    //             continue;
    //         });
    //         current_line += 1;
    //         if current_line % MAX_NGRAMS == 0 || current_line >= cache_lines.len() as u32 {
    //             break;
    //         }
    //     }
    //     print_verbose("Generating ngrams");
    //     // let data_ngram = generate_ngrams(3, &tmp);
    //     let data_ngram = generate_ngrams_bytes(NGRAM_SIZE, &tmp);
    //     print_verbose("Done");
    //     for ngram in &input_ngrams {
    //         if let Some(val) = data_ngram.get(ngram) {
    //             for entry in val {
    //                 let e = amount_matching_ngrams.entry((*entry).clone()).or_insert(0);
    //                 *e += 1;
    //             }
    //         }
    //     }
    //     if current_line >= cache_lines.len() as u32 {
    //         break;
    //     }
    // }

    // Multithreading ngram creation
    let mut handles = vec![];
    // Spawning threads
    for _thread_id in 0..config.thread_count {
        let handle = thread::spawn({
            let line_queue = line_queue.clone();
            let input_ngrams = input_ngrams.clone();
            let amount_matching_ngrams = amount_matching_ngrams.clone();
            move || {
                let mut start_vec: Vec<String>;
                loop {

                    {
                        let mut queue = line_queue.lock().unwrap();
                        if !queue.is_empty() {
                            start_vec = match queue.pop_front() {
                                Some(val) => val,
                                None => break,
                            };
                        } else {
                            break;
                        }
                    }

                    let data_ngram = generate_ngrams_bytes(NGRAM_SIZE, &start_vec);
                    let mut map = amount_matching_ngrams.lock().unwrap();
                    for ngram in &input_ngrams {
                        if let Some(val) = data_ngram.get(ngram) {
                            for entry in val {
                                let e = map.entry((*entry).clone()).or_insert(0);
                                *e += 1;
                            }
                        }
                    }
                    drop(map);

                }
            }
        });
        handles.push((_thread_id, handle));
    }

    for (id, handle) in handles {
        handle.join().unwrap();
        print_verbose(format!("Thread ID {} done", id).as_str());
    }

    let sort_key_val: Arc<Mutex<HashMap<String, i32>>> = Arc::new(Mutex::new(HashMap::new()));
    let handle = thread::spawn({
        let amount_matching_ngrams = amount_matching_ngrams.lock().unwrap().clone();
        let sort_key_val = sort_key_val.clone();
        let input = input.clone();
        move || {
            for (k, _) in &amount_matching_ngrams {
                let lev_dist = filename_lev_distance(&k, &input);
                let mut map = sort_key_val.lock().unwrap(); // Get mutex lock
                // TODO: do more testing on this
                let mut matching_ngrams = *amount_matching_ngrams.get(k).unwrap_or(&0);
                if matching_ngrams < NGRAM_MIN_MATCHES {
                    matching_ngrams = 0;
                }
                map.insert(
                    k.clone(),
                    (lev_dist * FILENAME_WEIGHT) as i32
                        - (matching_ngrams * NGRAM_WEIGHT) as i32,
                );
            }
        }
    });

    handle.join().unwrap();
    print_verbose("Thread done");
    print_verbose("Done creating ngrams, now filling result");
    let amount_matching_ngrams_clone = amount_matching_ngrams.lock().unwrap().clone();
    for (k, _) in &amount_matching_ngrams_clone {
        // let lev_dist = lev_dist_v2(&k, &input);
        result.push((&k, *sort_key_val.lock().unwrap().get(k).unwrap()));
    }
    print_verbose("Result fill done");
    print_verbose("Now sorting result");
    result.sort_unstable_by_key(|e| {
        e.1
        // e.1 + sort_key_val.lock().unwrap().get(e.0).unwrap()
        // e.1 - (*amount_matching_ngrams.get(e.0).unwrap_or(&0) * NGRAM_WEIGHT) as i32 + (filename_lev_distance(&e.0, input) * FILENAME_WEIGHT) as i32
    });

    let mut output_strings = vec![];
    let mut c = 0;
    for res in &result {
        output_strings.push(
            format!(
                "{}, {}",
                res.0,
                res.1 // res.1 - (*amount_matching_ngrams.get(res.0).unwrap_or(&0) * NGRAM_WEIGHT) as i32 + (filename_lev_distance(&res.0, input) * FILENAME_WEIGHT) as i32
                )
            );
        c += 1;
        if c == config.max_results {
            output_strings.push(format!("+ {} results.\n...", result.len() - config.max_results as usize));
            break;
        }
    }
    output_strings.reverse();
    for out in output_strings {
        println!("{}", out);
    }
}

fn filename_lev_distance(full_path: &String, search_term: &String) -> u32 {
    let path_split = full_path.split('/').collect::<Vec<&str>>();
    let file_name = path_split.get(path_split.len() - 1).unwrap();
    lev_dist_v2(&file_name.to_string(), search_term)
}

fn generate_ngrams<'a>(size: u32, vec: &Vec<&'a String>) -> HashMap<String, Vec<&'a String>> {
    let mut hmap: HashMap<String, Vec<&String>> = HashMap::new();
    for entry in vec {
        // println!("{}", entry);
        if *entry == "" {
            continue;
        }
        for i in 0..(entry.len() - size as usize) {
            let ngram = match entry.get((0 + i)..(size as usize + i)) {
                Some(val) => val.to_string(),
                None => {
                    eprintln!("error creating ngram for {}", entry);
                    continue;
                }
            };
            let e = hmap.entry(ngram).or_insert(vec![]);
            e.push(entry);
        }
    }
    hmap
}

/// Generates ngrams from byte slices.
///
/// Returns a `HashMap<Vev<u8>, Vec<&String>>`.
/// The `&String`'s has the same lifetime as the ones
/// in the `Vec` that is sent in to the function.
fn generate_ngrams_bytes<'a>(
    size: u32,
    vec: &Vec<String>,
) -> HashMap<Vec<u8>, Vec<&String>> {
    let mut hmap: HashMap<Vec<u8>, Vec<&String>> = HashMap::new();
    for entry in vec {
        if *entry == "" {
            continue;
        }
        let size_usize = size as usize;
        for i in 0..(entry.len() - size_usize) {
            let ngram = entry.as_bytes().get(i..(i + size_usize)).unwrap();
            let e = hmap
                .entry(ngram.iter().map(|b| b.to_owned()).collect::<Vec<u8>>())
                .or_insert(vec![]);
            e.push(entry);
        }
    }
    hmap
}

fn generate_ngram(size: u32, word: &String) -> Vec<String> {
    let mut rvec: Vec<String> = vec![];
    for i in 0..(word.len() - size as usize) {
        let ngram = match word.get((0 + i)..(size as usize + i)) {
            Some(val) => val.to_string(),
            None => continue,
        };
        rvec.push(ngram);
    }
    rvec
}

fn generate_ngram_bytes(size: u32, word: &String) -> Vec<Vec<u8>> {
    let mut rvec: Vec<Vec<u8>> = vec![];
    let size_usize = size as usize;
    for i in 0..(word.len() - size_usize) {
        let ngram = word.as_bytes().get(i..(i + size_usize)).unwrap();
        rvec.push(ngram.iter().map(|b| b.to_owned()).collect());
    }
    rvec
}

fn _lev_dist(s1: &String, s2: &String) -> usize {
    let mut r_matrix: Vec<Vec<usize>> = vec![vec![0; s1.len() + 1]; s2.len() + 1];

    for y in 0..r_matrix.len() {
        r_matrix[y][0] = y;
    }
    for x in 0..r_matrix[0].len() {
        r_matrix[0][x] = x;
    }

    for j in 1..r_matrix[0].len() {
        for i in 1..r_matrix.len() {
            let s_cost: usize;
            if s1.chars().nth(j - 1) == s2.chars().nth(i - 1) {
                s_cost = 0;
            } else {
                s_cost = 1;
            }

            r_matrix[i][j] = min(
                r_matrix[i - 1][j] + 1,
                min(r_matrix[i][j - 1] + 1, r_matrix[i - 1][j - 1] + s_cost),
            );
        }
    }

    r_matrix[s2.len()][s1.len()]
}

fn lev_dist_v2(s: &String, t: &String) -> u32 {
    let n = t.len() as u32;
    let m = s.len() as u32;

    let mut v0: Vec<u32> = vec![0; (n + 1) as usize];
    let mut v1: Vec<u32> = vec![0; (n + 1) as usize];

    let mut counter = 0;
    v0.iter_mut().for_each(|e| {
        *e = counter;
        counter += 1;
    });

    for i in 0..m {
        v1[0] = i + 1;

        let s_nth = match s.chars().nth(i as usize) {
            Some(char) => char,
            None => '\0',
        };

        for j in 0..t.len() {
            // n - 1
            let del_cost = v0.get(j + 1).unwrap() + 1;
            let ins_cost = v1.get(j).unwrap() + 1;
            let sub_cost: u32;
            let t_nth = match t.chars().nth(j) {
                Some(char) => char,
                None => '\0',
            };
            if s_nth == t_nth && s_nth != '\0' {
                sub_cost = *v0.get(j).unwrap();
            } else {
                sub_cost = *v0.get(j).unwrap() + 1;
            }
            v1[j + 1] = min(del_cost, min(ins_cost, sub_cost));
        }
        v0 = v1.clone();
    }
    v0[t.len()]
}
