use once_cell::sync::OnceCell;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::mem;

use super::NodeDictionary;

#[derive(PartialEq, Debug, Clone)]
pub struct TripleStore {
    elem: Vec<Chunk>,
    size: usize,
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Chunk {
    so: Vec<[u64; 2]>,
    os: OnceCell<Vec<[u64; 2]>>,
    so_dirty: bool,
}

impl Chunk {
    // # Pre-condition
    // so must be sorted.
    #[cfg_attr(debug_assertions, flamer::flame)]
    fn new(so: Vec<[u64; 2]>) -> Chunk {
        #[cfg(debug_assertions)]
        {
            for i in 1..so.len() {
                assert!(so[i] >= so[i - 1]);
            }
        }
        Chunk {
            so,
            os: OnceCell::new(),
            so_dirty: false,
        }
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    fn so_sort(&mut self) -> usize {
        if self.so_dirty {
            self.so_dirty = false;
            self.os = OnceCell::new();
            bucket_sort_pairs(&mut self.so)
        } else {
            self.so.len()
        }
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn so(&self) -> &Vec<[u64; 2]> {
        #[cfg(debug_assertions)]
        debug_assert!(!self.so_dirty);
        &self.so
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    fn so_unsorted(&self) -> &Vec<[u64; 2]> {
        &self.so
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn os(&self) -> &Vec<[u64; 2]> {
        #[cfg(debug_assertions)]
        debug_assert!(!self.so_dirty);
        self.os.get_or_init(|| {
            let mut v = self.so.clone();
            if !v.is_empty() {
                bucket_sort_pairs_os(&mut v);
            }
            v
        })
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    fn res_to_prop(&mut self, res: u64, prop: u64) {
        for pair in self.so.iter_mut() {
            for i in 0..=1 {
                if pair[i] == res {
                    pair[i] = prop;
                }
            }
        }
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    fn add_so(&mut self, so: [u64; 2]) {
        self.so_dirty = true;
        self.so.push(so);
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    fn add_sos(&mut self, sos: &Vec<[u64; 2]>) {
        self.so_dirty = true;
        self.so.extend(sos);
    }
}

impl TripleStore {
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn new() -> Self {
        let elem = Vec::new();
        Self { elem, size: 0 }
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    #[inline]
    pub fn elem(&self) -> &Vec<Chunk> {
        &self.elem
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    #[inline]
    pub fn elem_mut(&mut self) -> &mut Vec<Chunk> {
        &mut self.elem
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    #[inline]
    pub fn set_elem(&mut self, elem: Vec<Chunk>) {
        self.elem = elem;
    }
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn add_triple(&mut self, triple: [u64; 3]) {
        let [is, ip, io] = triple;
        let ip_to_store = NodeDictionary::prop_idx_to_idx(ip);
        if ip_to_store >= self.elem.len() {
            self.elem.resize_with(ip_to_store + 1, Default::default);
        }
        self.add_triple_raw(is, ip_to_store, io);
    }
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn add_all(&mut self, other: Self) {
        if other.elem.len() > self.elem.len() {
            self.elem.resize_with(other.elem.len(), Default::default);
        }
        for ip in 0..other.elem.len() {
            let triples_to_add = other.elem[ip].so_unsorted();
            self.add_triples_raw(ip, triples_to_add);
        }
    }
    /// # Pre-condition
    /// `self.elem` must have an element at index `ip`
    #[inline]
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn add_triple_raw(&mut self, is: u64, ip: usize, io: u64) {
        self.size += 1;
        self.elem[ip].add_so([is, io]);
    }
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn add_triples(&mut self, ip: u64, sos: &Vec<[u64; 2]>) {
        let ip_to_store = NodeDictionary::prop_idx_to_idx(ip);
        if ip_to_store >= self.elem.len() {
            self.elem.resize_with(ip_to_store + 1, Default::default);
        }
        self.add_triples_raw(ip_to_store, sos);
    }
    /// # Pre-condition
    /// `self.elem` must have an element at index `ip`
    #[inline]
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn add_triples_raw(&mut self, ip: usize, sos: &Vec<[u64; 2]>) {
        self.size += sos.len();
        self.elem[ip].add_sos(sos);
    }
    #[cfg(not(debug_assertions))]
    pub fn sort(&mut self) {
        if self.elem.is_empty() {
            return;
        }
        self.size = self.elem.par_iter_mut().map(|chunk| chunk.so_sort()).sum();
    }
    #[cfg(debug_assertions)]
    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn sort(&mut self) {
        if self.elem.is_empty() {
            return;
        }
        self.size = self.elem.iter_mut().map(|chunk| chunk.so_sort()).sum();
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn res_to_prop(&mut self, res: u64, prop: u32) {
        for chunk in &mut self.elem {
            chunk.res_to_prop(res, prop.into());
        }
        /////////
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn size(&mut self) -> usize {
        self.size
    }

    #[cfg_attr(debug_assertions, flamer::flame)]
    pub fn join(a: &Self, b: &Self) -> Self {
        let len = std::cmp::max(a.elem.len(), b.elem.len());
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut size = 0;
        if b.elem.is_empty() {
            chunks = a.elem.clone();
            size = a.size;
        } else {
            for i in 0..len {
                let mut new_so: Vec<[u64; 2]> = Vec::new();
                let mut original = &vec![];
                let mut infered = &vec![];
                if let Some(opt) = a.elem.get(i) {
                    original = opt.so();
                }
                if let Some(opt) = b.elem.get(i) {
                    infered = opt.so();
                }
                match (original.is_empty(), infered.is_empty()) {
                    (false, false) => {
                        let mut index_o = 0;
                        let mut index_i = 0;
                        let mut new_o = true;
                        let mut new_i = true;
                        let mut s_o = 0;
                        let mut o_o = 0;
                        let mut s_i = 0;
                        let mut o_i = 0;
                        while index_o < original.len() || index_i < infered.len() {
                            if new_o {
                                if index_o < original.len() {
                                    let pair_o = original[index_o];
                                    s_o = pair_o[0];
                                    o_o = pair_o[1];
                                } else {
                                    s_o = u64::max_value();
                                    o_o = u64::max_value();
                                }
                                index_o += 1;
                            }
                            if new_i {
                                if index_i < infered.len() {
                                    let pair_i = infered[index_i];
                                    s_i = pair_i[0];
                                    o_i = pair_i[1];
                                } else {
                                    s_i = u64::max_value();
                                    o_i = u64::max_value();
                                }
                                index_i += 1;
                            }
                            new_o = false;
                            new_i = false;
                            match s_o.cmp(&s_i) {
                                Ordering::Less => {
                                    new_so.push([s_o, o_o]);
                                    new_o = true;
                                }
                                Ordering::Greater => {
                                    new_so.push([s_i, o_i]);
                                    new_i = true;
                                }
                                Ordering::Equal => match o_o.cmp(&o_i) {
                                    Ordering::Less => {
                                        new_so.push([s_o, o_o]);
                                        new_o = true;
                                    }
                                    Ordering::Greater => {
                                        new_so.push([s_i, o_i]);
                                        new_i = true;
                                    }
                                    Ordering::Equal => {
                                        new_so.push([s_o, o_o]);
                                        new_o = true;
                                        new_i = true;
                                    }
                                },
                            }
                        }
                    }
                    (false, true) => {
                        new_so = original.clone();
                    }
                    (true, false) => {
                        new_so = infered.clone();
                    }
                    (true, true) => (),
                }
                size += new_so.len();
                chunks.push(Chunk::new(new_so));
            }
        }
        Self { elem: chunks, size }
    }
}

/// Sort the pairs and remove duplicates
#[cfg_attr(debug_assertions, flamer::flame)]
fn bucket_sort_pairs(pairs: &mut Vec<[u64; 2]>) -> usize {
    if pairs.is_empty() {
        return 0;
    }
    let (min, max) = pairs
        .iter()
        .map(|pair| pair[0])
        .fold((u64::max_value(), 0), |acc, x| (acc.0.min(x), acc.1.max(x)));
    let width = (max - min + 1) as usize;
    let mut hist: Vec<usize> = vec![0; width];
    let mut hist2: Vec<usize> = Vec::with_capacity(width);
    let mut cumul: Vec<usize> = vec![0; width];

    build_hist(pairs, min, 0, &mut hist);
    mem::replace(&mut hist2, hist.to_vec());
    build_cumul(&hist, &mut cumul);
    let len = pairs.len();
    let mut objects = vec![0; len];
    for i in 0..len {
        let val = pairs[i];
        let val_s = val[0];
        let val_o = val[1];

        let idx = (val_s - min) as usize;

        let pos = cumul[idx];
        let remaining = hist[idx];

        let obj_idx = (pos + remaining - 1) as usize;

        hist[idx] -= 1;
        objects[obj_idx] = val_o;
    }

    for i in 0..(width - 1) {
        insertion_sort_slice(&mut objects, cumul[i], cumul[i + 1]);
    }
    insertion_sort_slice(&mut objects, cumul[width - 1], len);

    let mut j = 0;
    let mut l = 0;
    let mut last = 0;
    for i in 0..width {
        let val = hist2[i];
        let s = min + i as u64;
        for k in 0..val {
            let o = objects[l];
            l += 1;
            if k == 0 || o != last {
                pairs[j] = [s, o];
                j += 1;
            }
            last = o;
        }
    }

    pairs.truncate(j);

    j
}

#[cfg_attr(debug_assertions, flamer::flame)]
fn insertion_sort_slice(v: &mut [u64], from: usize, to: usize) {
    for i in from..to {
        let mut j = i;
        let tmp = v[i];
        while j > from && v[j - 1] > tmp {
            v[j] = v[j - 1];
            j -= 1;
        }
        v[j] = tmp;
    }
}

#[inline]
#[cfg_attr(debug_assertions, flamer::flame)]
fn build_hist(pairs: &[[u64; 2]], min: u64, pair_elem: usize, hist: &mut [usize]) {
    for pair in pairs {
        hist[(pair[pair_elem] - min) as usize] += 1;
    }
}

#[inline]
#[cfg_attr(debug_assertions, flamer::flame)]
fn build_cumul(hist: &[usize], cumul: &mut [usize]) {
    for i in 1..hist.len() {
        cumul[i] = cumul[i - 1] + hist[i - 1];
    }
}

/// Reverse the pairs and sort them
#[cfg_attr(debug_assertions, flamer::flame)]
fn bucket_sort_pairs_os(pairs: &mut Vec<[u64; 2]>) {
    let (min, max) = pairs
        .iter()
        .map(|pair| pair[1])
        .fold((u64::max_value(), 0), |acc, x| (acc.0.min(x), acc.1.max(x)));
    let width = (max - min + 1) as usize;
    let mut hist: Vec<usize> = vec![0; width];
    let mut hist2: Vec<usize> = Vec::with_capacity(width);
    let mut cumul: Vec<usize> = vec![0; width];
    build_hist(pairs, min, 1, &mut hist);
    mem::replace(&mut hist2, hist.to_vec());
    build_cumul(&hist, &mut cumul);
    let len = pairs.len();
    let mut objects = vec![0; len];
    for i in 0..len {
        let val = pairs[i];
        let val_s = val[0];
        let val_o = val[1];

        let idx = (val_o - min) as usize;

        let pos = cumul[idx];
        let remaining = hist[idx];

        let obj_idx = (pos + remaining - 1) as usize;

        hist[idx] -= 1;
        objects[obj_idx] = val_s;
    }

    for i in 0..(width - 1) {
        insertion_sort_slice(&mut objects, cumul[i], cumul[i + 1]);
    }
    insertion_sort_slice(&mut objects, cumul[width - 1], len);

    let mut j = 0;
    let mut l = 0;
    for i in 0..width {
        let val = hist2[i];
        for _ in 0..val {
            let s = objects[l];
            l += 1;
            let o = min + i as u64;
            pairs[j] = [o, s];
            j += 1;
        }
    }
}

#[test]
#[cfg_attr(debug_assertions, flamer::flame)]
fn test_sort() {
    let mut pairs = vec![[2, 1], [1, 3]];
    bucket_sort_pairs(&mut pairs);
    let expected = [[1, 3], [2, 1]];
    assert_eq!(pairs, expected);
    let mut pairs = vec![[2, 1], [1, 3], [2, 1]];
    bucket_sort_pairs(&mut pairs);
    let expected = [[1, 3], [2, 1]];
    assert_eq!(pairs, expected);
    let mut pairs = vec![[2, 1], [1, 3], [1, 3]];
    bucket_sort_pairs(&mut pairs);
    let expected = [[1, 3], [2, 1]];
    assert_eq!(pairs, expected);
    let mut pairs = vec![[2, 3], [2, 1]];
    bucket_sort_pairs(&mut pairs);
    let expected = [[2, 1], [2, 3]];
    assert_eq!(pairs, expected);
}

#[test]
#[cfg_attr(debug_assertions, flamer::flame)]
fn test_join() {
    let a = TripleStore {
        elem: vec![Chunk::default()],
        size: 0,
    };
    let b = TripleStore {
        elem: vec![Chunk::default()],
        size: 0,
    };
    let expected = TripleStore {
        elem: vec![Chunk::default()],
        size: 0,
    };
    assert_eq!(TripleStore::join(&a, &b), expected);
    let a = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    let b = TripleStore {
        elem: vec![Chunk::default()],
        size: 0,
    };
    let expected = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    assert_eq!(TripleStore::join(&a, &b), expected);
    let a = TripleStore {
        elem: vec![Chunk::default()],
        size: 0,
    };
    let b = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    let expected = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    assert_eq!(TripleStore::join(&a, &b), expected);
    let a = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    let b = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    let expected = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    assert_eq!(TripleStore::join(&a, &b), expected);
    let a = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1], [1, 2]])],
        size: 2,
    };
    let b = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1]])],
        size: 1,
    };
    let expected = TripleStore {
        elem: vec![Chunk::new(vec![[1, 1], [1, 2]])],
        size: 2,
    };
    assert_eq!(TripleStore::join(&a, &b), expected);
}
