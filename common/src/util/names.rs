use rand::{thread_rng, Rng};

const NAMES: [&str; 24] = [
    "Olaf",
    "Günter",
    "Tom",
    "Jerry",
    "Tim",
    "Jacob",
    "Edward",
    "Jack",
    "Daniel",
    "Wolfgang",
    "Simone",
    "May",
    "Dieter",
    "Lisa",
    "Catherine",
    "Lydia",
    "Kevin",
    "Gemma",
    "Alex",
    "Eun",
    "Sariyah",
    "Chung",
    "Lauren",
    "Paramita",
];

pub fn generate() -> &'static str {
    // unwrap is safe, choose would return None only if NAMES was empty.
    thread_rng().choose(&NAMES).unwrap()
}
