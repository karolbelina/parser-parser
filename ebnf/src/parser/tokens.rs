use super::{Span, Spanned, Token};
use nom::{
    Compare, CompareResult, FindSubstring, FindToken, InputIter, InputLength, InputTake, Needed,
    Slice, UnspecializedInput,
};
use std::{
    iter::{Cloned, Enumerate},
    ops::{Range, RangeFrom, RangeFull, RangeTo},
    slice::Iter,
};

#[derive(Debug, Clone)]
pub struct Tokens<'a> {
    inner: &'a [Spanned<Token>],
    last_span: Span,
}

impl<'a> Tokens<'a> {
    pub fn new(tokens: &'a [Spanned<Token>]) -> Tokens<'a> {
        let first_span = match tokens.iter().next() {
            Some(token) => token.span,
            None => Span::new(),
        };
        Tokens {
            inner: tokens,
            last_span: first_span,
        }
    }

    pub fn last_span(&self) -> Span {
        self.last_span
    }
}

impl<'a> PartialEq for Tokens<'a> {
    fn eq(&self, other: &Tokens) -> bool {
        self.inner == other.inner
    }
}

impl<'a> Eq for Tokens<'a> {}

impl<'a> InputLength for Tokens<'a> {
    fn input_len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> InputIter for Tokens<'a> {
    type Item = Spanned<Token>;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Cloned<Iter<'a, Self::Item>>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.inner.iter().cloned()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.inner.iter().position(|t| predicate(t.clone()))
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.inner.len() >= count {
            Ok(count)
        } else {
            Err(Needed::Unknown)
        }
    }
}

impl<'a> InputTake for Tokens<'a>
where
    Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
{
    fn take(&self, count: usize) -> Self {
        Tokens {
            inner: &self.inner[0..count],
            last_span: self.last_span,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.inner.split_at(count);
        let last_span = if count > 0 {
            self.inner[count - 1].span
        } else {
            self.last_span
        };
        (
            Tokens {
                inner: suffix,
                last_span: self.last_span,
            },
            Tokens {
                inner: prefix,
                last_span,
            },
        )
    }
}

impl<'a> UnspecializedInput for Tokens<'a> {}

impl<'a> Compare<Tokens<'a>> for Tokens<'a> {
    #[inline(always)]
    fn compare(&self, other: Tokens<'a>) -> CompareResult {
        let pos = self
            .inner
            .iter()
            .zip(other.inner.iter())
            .position(|(a, b)| a != b);

        match pos {
            Some(_) => CompareResult::Error,
            None => {
                if self.inner.len() >= other.inner.len() {
                    CompareResult::Ok
                } else {
                    CompareResult::Incomplete
                }
            }
        }
    }

    #[inline(always)]
    fn compare_no_case(&self, other: Tokens<'a>) -> CompareResult {
        self.compare(other)
    }
}

impl<'a> Slice<Range<usize>> for Tokens<'a> {
    fn slice(&self, range: Range<usize>) -> Tokens<'a> {
        let last_span = if range.start > 0 {
            self.inner[range.start - 1].span
        } else {
            self.last_span
        };
        Tokens {
            inner: &self.inner[range],
            last_span,
        }
    }
}

impl<'a> Slice<RangeTo<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Tokens<'a> {
        Tokens {
            inner: &self.inner[range],
            last_span: self.last_span,
        }
    }
}

impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Tokens<'a> {
        let last_span = if range.start > 0 {
            self.inner[range.start - 1].span
        } else {
            self.last_span
        };
        Tokens {
            inner: &self.inner[range],
            last_span,
        }
    }
}

impl<'a> Slice<RangeFull> for Tokens<'a> {
    fn slice(&self, range: RangeFull) -> Tokens<'a> {
        Tokens {
            inner: &self.inner[range],
            last_span: self.last_span,
        }
    }
}

impl<'a> FindToken<Spanned<Token>> for Tokens<'a> {
    fn find_token(&self, token: Spanned<Token>) -> bool {
        self.inner.contains(&token)
    }
}

impl<'a> FindSubstring<&'a [Spanned<Token>]> for Tokens<'a> {
    #[inline]
    fn find_substring(&self, substr: &'a [Spanned<Token>]) -> Option<usize> {
        let substr_len = substr.len();

        if substr_len == 0 {
            Some(0)
        } else if substr_len == 1 {
            self.inner.iter().position(|t| t == &substr[0])
        } else if substr_len > self.inner.len() {
            None
        } else {
            let max = self.inner.len() - substr_len;
            let mut offset = 0;
            let mut haystack = &self.inner[..];

            while let Some(position) = haystack.iter().position(|t| t == &substr[0]) {
                offset += position;

                if offset > max {
                    return None;
                }

                if &haystack[position..position + substr_len] == substr {
                    return Some(offset);
                }

                haystack = &haystack[position + 1..];
                offset += 1;
            }

            None
        }
    }
}
