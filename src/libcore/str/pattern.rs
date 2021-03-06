// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use prelude::*;

// Pattern

/// A string pattern.
///
/// A `Pattern<'a>` expresses that the implementing type
/// can be used as a string pattern for searching in a `&'a str`.
///
/// For example, both `'a'` and `"aa"` are patterns that
/// would match at index `1` in the string `"baaaab"`.
///
/// The trait itself acts as a builder for an associated
/// `Searcher` type, which does the actual work of finding
/// occurrences of the pattern in a string.
pub trait Pattern<'a>: Sized {
    /// Associated searcher for this pattern
    type Searcher: Searcher<'a>;

    /// Construct the associated searcher from
    /// `self` and the `haystack` to search in.
    fn into_searcher(self, haystack: &'a str) -> Self::Searcher;

    /// Check whether the pattern matches anywhere in the haystack
    #[inline]
    fn is_contained_in(self, haystack: &'a str) -> bool {
        self.into_searcher(haystack).next_match().is_some()
    }

    /// Check whether the pattern matches at the front of the haystack
    #[inline]
    fn is_prefix_of(self, haystack: &'a str) -> bool {
        match self.into_searcher(haystack).next() {
            SearchStep::Match(0, _) => true,
            _ => false,
        }
    }

    /// Check whether the pattern matches at the back of the haystack
    #[inline]
    fn is_suffix_of(self, haystack: &'a str) -> bool
        where Self::Searcher: ReverseSearcher<'a>
    {
        match self.into_searcher(haystack).next_back() {
            SearchStep::Match(_, j) if haystack.len() == j => true,
            _ => false,
        }
    }
}

// Searcher

/// Result of calling `Searcher::next()` or `ReverseSearcher::next_back()`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SearchStep {
    /// Expresses that a match of the pattern has been found at
    /// `haystack[a..b]`.
    Match(usize, usize),
    /// Expresses that `haystack[a..b]` has been rejected as a possible match
    /// of the pattern.
    ///
    /// Note that there might be more than one `Reject` between two `Match`es,
    /// there is no requirement for them to be combined into one.
    Reject(usize, usize),
    /// Expresses that every byte of the haystack has been visted, ending
    /// the iteration.
    Done
}

/// A searcher for a string pattern.
///
/// This trait provides methods for searching for non-overlapping
/// matches of a pattern starting from the front (left) of a string.
///
/// It will be implemented by associated `Searcher`
/// types of the `Pattern` trait.
///
/// The trait is marked unsafe because the indices returned by the
/// `next()` methods are required to lie on valid utf8 boundaries in
/// the haystack. This enables consumers of this trait to
/// slice the haystack without additional runtime checks.
pub unsafe trait Searcher<'a> {
    /// Getter for the underlaying string to be searched in
    ///
    /// Will always return the same `&str`
    fn haystack(&self) -> &'a str;

    /// Performs the next search step starting from the front.
    ///
    /// - Returns `Match(a, b)` if `haystack[a..b]` matches the pattern.
    /// - Returns `Reject(a, b)` if `haystack[a..b]` can not match the
    ///   pattern, even partially.
    /// - Returns `Done` if every byte of the haystack has been visited
    ///
    /// The stream of `Match` and `Reject` values up to a `Done`
    /// will contain index ranges that are adjacent, non-overlapping,
    /// covering the whole haystack, and laying on utf8 boundaries.
    ///
    /// A `Match` result needs to contain the whole matched pattern,
    /// however `Reject` results may be split up into arbitrary
    /// many adjacent fragments. Both ranges may have zero length.
    ///
    /// As an example, the pattern `"aaa"` and the haystack `"cbaaaaab"`
    /// might produce the stream
    /// `[Reject(0, 1), Reject(1, 2), Match(2, 5), Reject(5, 8)]`
    fn next(&mut self) -> SearchStep;

    /// Find the next `Match` result. See `next()`
    #[inline]
    fn next_match(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next() {
                SearchStep::Match(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }

    /// Find the next `Reject` result. See `next()`
    #[inline]
    fn next_reject(&mut self) -> Option<(usize, usize)> {
        loop {
            match self.next() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }
}

/// A reverse searcher for a string pattern.
///
/// This trait provides methods for searching for non-overlapping
/// matches of a pattern starting from the back (right) of a string.
///
/// It will be implemented by associated `Searcher`
/// types of the `Pattern` trait if the pattern supports searching
/// for it from the back.
///
/// The index ranges returned by this trait are not required
/// to exactly match those of the forward search in reverse.
///
/// For the reason why this trait is marked unsafe, see them
/// parent trait `Searcher`.
pub unsafe trait ReverseSearcher<'a>: Searcher<'a> {
    /// Performs the next search step starting from the back.
    ///
    /// - Returns `Match(a, b)` if `haystack[a..b]` matches the pattern.
    /// - Returns `Reject(a, b)` if `haystack[a..b]` can not match the
    ///   pattern, even partially.
    /// - Returns `Done` if every byte of the haystack has been visited
    ///
    /// The stream of `Match` and `Reject` values up to a `Done`
    /// will contain index ranges that are adjacent, non-overlapping,
    /// covering the whole haystack, and laying on utf8 boundaries.
    ///
    /// A `Match` result needs to contain the whole matched pattern,
    /// however `Reject` results may be split up into arbitrary
    /// many adjacent fragments. Both ranges may have zero length.
    ///
    /// As an example, the pattern `"aaa"` and the haystack `"cbaaaaab"`
    /// might produce the stream
    /// `[Reject(7, 8), Match(4, 7), Reject(1, 4), Reject(0, 1)]`
    fn next_back(&mut self) -> SearchStep;

    /// Find the next `Match` result. See `next_back()`
    #[inline]
    fn next_match_back(&mut self) -> Option<(usize, usize)>{
        loop {
            match self.next_back() {
                SearchStep::Match(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }

    /// Find the next `Reject` result. See `next_back()`
    #[inline]
    fn next_reject_back(&mut self) -> Option<(usize, usize)>{
        loop {
            match self.next_back() {
                SearchStep::Reject(a, b) => return Some((a, b)),
                SearchStep::Done => return None,
                _ => continue,
            }
        }
    }
}

/// A marker trait to express that a `ReverseSearcher`
/// can be used for a `DoubleEndedIterator` implementation.
///
/// For this, the impl of `Searcher` and `ReverseSearcher` need
/// to follow these conditions:
///
/// - All results of `next()` need to be identical
///   to the results of `next_back()` in reverse order.
/// - `next()` and `next_back()` need to behave as
///   the two ends of a range of values, that is they
///   can not "walk past each other".
///
/// # Examples
///
/// `char::Searcher` is a `DoubleEndedSearcher` because searching for a
/// `char` only requires looking at one at a time, which behaves the same
/// from both ends.
///
/// `(&str)::Searcher` is not a `DoubleEndedSearcher` because
/// the pattern `"aa"` in the haystack `"aaa"` matches as either
/// `"[aa]a"` or `"a[aa]"`, depending from which side it is searched.
pub trait DoubleEndedSearcher<'a>: ReverseSearcher<'a> {}

// Impl for a CharEq wrapper

#[doc(hidden)]
trait CharEq {
    fn matches(&mut self, char) -> bool;
    fn only_ascii(&self) -> bool;
}

impl CharEq for char {
    #[inline]
    fn matches(&mut self, c: char) -> bool { *self == c }

    #[inline]
    fn only_ascii(&self) -> bool { (*self as u32) < 128 }
}

impl<F> CharEq for F where F: FnMut(char) -> bool {
    #[inline]
    fn matches(&mut self, c: char) -> bool { (*self)(c) }

    #[inline]
    fn only_ascii(&self) -> bool { false }
}

impl<'a> CharEq for &'a [char] {
    #[inline]
    fn matches(&mut self, c: char) -> bool {
        self.iter().any(|&m| { let mut m = m; m.matches(c) })
    }

    #[inline]
    fn only_ascii(&self) -> bool {
        self.iter().all(|m| m.only_ascii())
    }
}

struct CharEqPattern<C: CharEq>(C);

struct CharEqSearcher<'a, C: CharEq> {
    char_eq: C,
    haystack: &'a str,
    char_indices: super::CharIndices<'a>,
    #[allow(dead_code)]
    ascii_only: bool,
}

impl<'a, C: CharEq> Pattern<'a> for CharEqPattern<C> {
    type Searcher = CharEqSearcher<'a, C>;

    #[inline]
    fn into_searcher(self, haystack: &'a str) -> CharEqSearcher<'a, C> {
        CharEqSearcher {
            ascii_only: self.0.only_ascii(),
            haystack: haystack,
            char_eq: self.0,
            char_indices: haystack.char_indices(),
        }
    }
}

unsafe impl<'a, C: CharEq> Searcher<'a> for CharEqSearcher<'a, C> {
    #[inline]
    fn haystack(&self) -> &'a str {
        self.haystack
    }

    #[inline]
    fn next(&mut self) -> SearchStep {
        let s = &mut self.char_indices;
        // Compare lengths of the internal byte slice iterator
        // to find length of current char
        let (pre_len, _) = s.iter.iter.size_hint();
        if let Some((i, c)) = s.next() {
            let (len, _) = s.iter.iter.size_hint();
            let char_len = pre_len - len;
            if self.char_eq.matches(c) {
                return SearchStep::Match(i, i + char_len);
            } else {
                return SearchStep::Reject(i, i + char_len);
            }
        }
        SearchStep::Done
    }
}

unsafe impl<'a, C: CharEq> ReverseSearcher<'a> for CharEqSearcher<'a, C> {
    #[inline]
    fn next_back(&mut self) -> SearchStep {
        let s = &mut self.char_indices;
        // Compare lengths of the internal byte slice iterator
        // to find length of current char
        let (pre_len, _) = s.iter.iter.size_hint();
        if let Some((i, c)) = s.next_back() {
            let (len, _) = s.iter.iter.size_hint();
            let char_len = pre_len - len;
            if self.char_eq.matches(c) {
                return SearchStep::Match(i, i + char_len);
            } else {
                return SearchStep::Reject(i, i + char_len);
            }
        }
        SearchStep::Done
    }
}

impl<'a, C: CharEq> DoubleEndedSearcher<'a> for CharEqSearcher<'a, C> {}

// Impl for &str

// Todo: Optimize the naive implementation here

#[derive(Clone)]
struct StrSearcher<'a, 'b> {
    haystack: &'a str,
    needle: &'b str,
    start: usize,
    end: usize,
    done: bool,
}

/// Non-allocating substring search.
///
/// Will handle the pattern `""` as returning empty matches at each utf8
/// boundary.
impl<'a, 'b> Pattern<'a> for &'b str {
    type Searcher = StrSearcher<'a, 'b>;

    #[inline]
    fn into_searcher(self, haystack: &'a str) -> StrSearcher<'a, 'b> {
        StrSearcher {
            haystack: haystack,
            needle: self,
            start: 0,
            end: haystack.len(),
            done: false,
        }
    }
}

unsafe impl<'a, 'b> Searcher<'a> for StrSearcher<'a, 'b>  {
    #[inline]
    fn haystack(&self) -> &'a str {
        self.haystack
    }

    #[inline]
    fn next(&mut self) -> SearchStep {
        str_search_step(self,
        |m: &mut StrSearcher| {
            // Forward step for empty needle
            let current_start = m.start;
            if !m.done {
                m.start = m.haystack.char_range_at(current_start).next;
            }
            SearchStep::Match(current_start, current_start)
        },
        |m: &mut StrSearcher| {
            // Forward step for nonempty needle
            let current_start = m.start;
            // Compare byte window because this might break utf8 boundaries
            let possible_match = &m.haystack.as_bytes()[m.start .. m.start + m.needle.len()];
            if possible_match == m.needle.as_bytes() {
                m.start += m.needle.len();
                SearchStep::Match(current_start, m.start)
            } else {
                // Skip a char
                let haystack_suffix = &m.haystack[m.start..];
                m.start += haystack_suffix.chars().next().unwrap().len_utf8();
                SearchStep::Reject(current_start, m.start)
            }
        })
    }
}

unsafe impl<'a, 'b> ReverseSearcher<'a> for StrSearcher<'a, 'b>  {
    #[inline]
    fn next_back(&mut self) -> SearchStep {
        str_search_step(self,
        |m: &mut StrSearcher| {
            // Backward step for empty needle
            let current_end = m.end;
            if !m.done {
                m.end = m.haystack.char_range_at_reverse(current_end).next;
            }
            SearchStep::Match(current_end, current_end)
        },
        |m: &mut StrSearcher| {
            // Backward step for nonempty needle
            let current_end = m.end;
            // Compare byte window because this might break utf8 boundaries
            let possible_match = &m.haystack.as_bytes()[m.end - m.needle.len() .. m.end];
            if possible_match == m.needle.as_bytes() {
                m.end -= m.needle.len();
                SearchStep::Match(m.end, current_end)
            } else {
                // Skip a char
                let haystack_prefix = &m.haystack[..m.end];
                m.end -= haystack_prefix.chars().rev().next().unwrap().len_utf8();
                SearchStep::Reject(m.end, current_end)
            }
        })
    }
}

// Helper function for encapsulating the common control flow
// of doing a search step from the front or doing a search step from the back
fn str_search_step<F, G>(mut m: &mut StrSearcher,
                         empty_needle_step: F,
                         nonempty_needle_step: G) -> SearchStep
    where F: FnOnce(&mut StrSearcher) -> SearchStep,
          G: FnOnce(&mut StrSearcher) -> SearchStep
{
    if m.done {
        SearchStep::Done
    } else if m.needle.len() == 0 && m.start <= m.end {
        // Case for needle == ""
        if m.start == m.end {
            m.done = true;
        }
        empty_needle_step(&mut m)
    } else if m.start + m.needle.len() <= m.end {
        // Case for needle != ""
        nonempty_needle_step(&mut m)
    } else if m.start < m.end {
        // Remaining slice shorter than needle, reject it
        m.done = true;
        SearchStep::Reject(m.start, m.end)
    } else {
        m.done = true;
        SearchStep::Done
    }
}

macro_rules! char_eq_pattern_impl {
    ($wrapper:ty, $wrapper_ident:ident) => {
        fn into_searcher(self, haystack: &'a str) -> $wrapper {
            $wrapper_ident(CharEqPattern(self).into_searcher(haystack))
        }
        #[inline]
        fn is_contained_in(self, haystack: &'a str) -> bool {
            CharEqPattern(self).is_contained_in(haystack)
        }
        #[inline]
        fn is_prefix_of(self, haystack: &'a str) -> bool {
            CharEqPattern(self).is_prefix_of(haystack)
        }
        #[inline]
        fn is_suffix_of(self, haystack: &'a str) -> bool
            where $wrapper: ReverseSearcher<'a>
        {
            CharEqPattern(self).is_suffix_of(haystack)
        }
    }
}

// Pattern for char

impl<'a> Pattern<'a> for char {
    type Searcher = CharSearcher<'a>;
    char_eq_pattern_impl!(CharSearcher<'a>, CharSearcher);
}

pub struct CharSearcher<'a>(CharEqSearcher<'a, char>);

unsafe impl<'a> Searcher<'a> for CharSearcher<'a> {
    #[inline]
    fn haystack(&self) -> &'a str { self.0.haystack() }
    #[inline]
    fn next(&mut self) -> SearchStep { self.0.next() }
}
unsafe impl<'a> ReverseSearcher<'a> for CharSearcher<'a> {
    #[inline]
    fn next_back(&mut self) -> SearchStep { self.0.next_back() }
}
impl<'a> DoubleEndedSearcher<'a> for CharSearcher<'a> {}

// Pattern for &[char]

impl<'a, 'b> Pattern<'a> for &'b [char] {
    type Searcher = CharSliceSearcher<'a, 'b>;
    char_eq_pattern_impl!(CharSliceSearcher<'a, 'b>, CharSliceSearcher);
}

pub struct CharSliceSearcher<'a, 'b>(CharEqSearcher<'a, &'b [char]>);

unsafe impl<'a, 'b> Searcher<'a> for CharSliceSearcher<'a, 'b> {
    #[inline]
    fn haystack(&self) -> &'a str { self.0.haystack() }
    #[inline]
    fn next(&mut self) -> SearchStep { self.0.next() }
}
unsafe impl<'a, 'b> ReverseSearcher<'a> for CharSliceSearcher<'a, 'b> {
    #[inline]
    fn next_back(&mut self) -> SearchStep { self.0.next_back() }
}
impl<'a, 'b> DoubleEndedSearcher<'a> for CharSliceSearcher<'a, 'b> {}

// Pattern for predicates

impl<'a, F: FnMut(char) -> bool> Pattern<'a> for F {
    type Searcher = CharPredSearcher<'a, F>;
    char_eq_pattern_impl!(CharPredSearcher<'a, F>, CharPredSearcher);
}

pub struct CharPredSearcher<'a, F: FnMut(char) -> bool>(CharEqSearcher<'a, F>);

unsafe impl<'a, F> Searcher<'a> for CharPredSearcher<'a, F>
    where F: FnMut(char) -> bool
{
    #[inline]
    fn haystack(&self) -> &'a str { self.0.haystack() }
    #[inline]
    fn next(&mut self) -> SearchStep { self.0.next() }
}
unsafe impl<'a, F> ReverseSearcher<'a> for CharPredSearcher<'a, F>
    where F: FnMut(char) -> bool
{
    #[inline]
    fn next_back(&mut self) -> SearchStep { self.0.next_back() }
}
impl<'a, F> DoubleEndedSearcher<'a> for CharPredSearcher<'a, F>
    where F: FnMut(char) -> bool
{}

// Pattern for &&str

impl<'a, 'b> Pattern<'a> for &'b &'b str {
    type Searcher = <&'b str as Pattern<'a>>::Searcher;
    #[inline]
    fn into_searcher(self, haystack: &'a str)
                     -> <&'b str as Pattern<'a>>::Searcher {
        (*self).into_searcher(haystack)
    }
    #[inline]
    fn is_contained_in(self, haystack: &'a str) -> bool {
        (*self).is_contained_in(haystack)
    }
    #[inline]
    fn is_prefix_of(self, haystack: &'a str) -> bool {
        (*self).is_prefix_of(haystack)
    }
    #[inline]
    fn is_suffix_of(self, haystack: &'a str) -> bool {
        (*self).is_suffix_of(haystack)
    }
}
