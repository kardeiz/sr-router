use std::boxed::Box;
use std::convert::{Into, AsMut};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
pub enum Segment {
    Static(String),
    Until(String),
    UntilEnd
}

impl Segment {

    pub fn parse<I: Into<String>>(x: I) -> Vec<Segment> {
        let x = x.into();
        let x = x.replace(r"\*", "%%ASTERISK%%");
        let mut out = Vec::new();
        let parts: Vec<_> = x.split("*").collect();
        
        for i in 0..parts.len() {
            let part = parts[i].replace("%%ASTERISK%%", "*");
            if i == 0 {
                if !part.is_empty() {
                    out.push(Segment::Static(part.into()));
                    continue;
                }
            }            
            if i == parts.len() - 1 {
                if part.is_empty() {
                    out.push(Segment::UntilEnd);
                    continue;
                }
            }
            out.push(Segment::Until(part.into()));
        }
        out
    }

    pub fn check(&self, check: &str, ind: usize) 
        -> Option<(usize, Option<(usize, usize)>)> {
        match *self {
            Segment::Static(ref s) => {
                if check[ind..].find(s) == Some(0) {
                    Some((ind + s.len(), None))
                } else {
                    None
                }
            },
            Segment::Until(ref s) => {
                if let Some(i) = check[ind..].find(s) {
                    Some((ind + s.len() + i, Some((ind, ind + i))))
                } else {
                    None
                }
            },
            Segment::UntilEnd => {
                if check[ind..].is_empty() { 
                    None
                } else {
                    Some((check.len(), Some((ind, check.len()))))
                }
                
            }
        }
    }

}

#[derive(Default, Debug)]
pub struct Root<T> { pub children: Vec<Node<T>> }

impl<T> Root<T> {

    pub fn new() -> Self { Root { children: Vec::new() } }

    pub fn check(&self, check: &str) 
        -> Option<(&T, Option<Vec<(usize, usize)>>)> {

        for child in &self.children {
            if let Some((t, mut opt_caps)) = child.check(check, 0) {
                if let Some(ref mut v) = opt_caps { 
                    v.reverse();
                }
                return Some((t, opt_caps));
            }
        }
        None
    }

    pub fn insert<I: Into<String>>(&mut self, x: I, val: T) -> &mut Self {
        let segments = Segment::parse(x);
        self.insert_inner(segments, val);
        self
    }

    pub fn insert_inner(&mut self, mut segments: Vec<Segment>, val: T) {

        if segments.is_empty() { return; }
        let curr = segments.remove(0);

        if segments.is_empty() {
            let mut leaf = LeafNode { segment: curr, target: val };
            self.children.push(Node::Leaf(leaf));
            return;
        }

        for child in self.children.iter_mut() {
            if let Some(child_as_branch) = child.as_mut_branch() {
                if &child_as_branch.segment == &curr {
                    child_as_branch.insert(segments, val);
                    return;
                }
            }
        }
        let mut branch = BranchNode { segment: curr, children: Vec::new() };
        branch.insert(segments, val);
        self.children.push(Node::Branch(branch));
    }

}

#[derive(Debug)]
pub struct BranchNode<T> { segment: Segment, children: Vec<Node<T>> }

#[derive(Debug)]
pub struct LeafNode<T> { segment: Segment, target: T }

impl<T> BranchNode<T> {

    pub fn insert(&mut self, mut segments: Vec<Segment>, val: T) {
        
        if segments.is_empty() { return; }
        
        let curr = segments.remove(0);
        
        if segments.is_empty() {
            let mut leaf = LeafNode { segment: curr, target: val };
            self.children.push(Node::Leaf(leaf));
            return;
        }

        for child in self.children.iter_mut() {
            if let Some(child_as_branch) = child.as_mut_branch() {
                if &child_as_branch.segment == &curr {
                    child_as_branch.insert(segments, val);
                    return;
                }
            }
        }

        let mut branch = BranchNode { segment: curr, children: Vec::new() };
        branch.insert(segments, val);
        self.children.push(Node::Branch(branch));
    }

    pub fn check(&self, check: &str, ind: usize) 
        -> Option<(&T, Option<Vec<(usize, usize)>>)> {

        let BranchNode { segment: ref s, children: ref children } = *self;

        if let Some((d1, opt_d2)) = s.check(check, ind) {
            for child in children {
                if let Some((t, mut opt_d3)) = child.check(check, d1) {
                    if let Some(ref mut d3) = opt_d3 {
                        if let Some(d2) = opt_d2 {
                            d3.push(d2);
                        }
                    }
                    return Some((t, opt_d3));
                }
            }
        }

        None
    }

}

impl<T> LeafNode<T> {

    pub fn check(&self, check: &str, ind: usize) 
        -> Option<(&T, Option<Vec<(usize, usize)>>)> {

        let LeafNode { segment: ref s, target: ref t } = *self;

        if let Some((d1, opt_d2)) = s.check(check, ind) {
            // Make sure the path is now empty
            if d1 == check.len() {
                let caps = opt_d2.map(|x| vec![x] );
                return Some((t, caps));
            }            
        } 

        None
    }

}


#[derive(Debug)]
pub enum Node<T> {
    Branch(BranchNode<T>),
    Leaf(LeafNode<T>)
}

impl<T> Node<T> {

    pub fn as_mut_branch<'a>(&'a mut self) -> Option<&'a mut BranchNode<T>> {
        match *self {
            Node::Branch(ref mut x) => Some(x),
            _ => None
        }
    }
    pub fn as_mut_leaf<'a>(&'a mut self) -> Option<&'a mut LeafNode<T>> {
        match *self {
            Node::Leaf(ref mut x) => Some(x),
            _ => None
        }
    }

    pub fn check(&self, check: &str, ind: usize) 
        -> Option<(&T, Option<Vec<(usize, usize)>>)> {
        match *self {
            Node::Branch(ref x) => x.check(check, ind),
            Node::Leaf(ref x) => x.check(check, ind)
        }
    }

}