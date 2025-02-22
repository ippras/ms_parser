use std::fmt::{self, Debug, Display, Formatter};

pub mod nom;
pub mod uom;

/// Preview
pub struct Preview<'a, T>(pub &'a [T]);

impl<T: Debug> Debug for Preview<'_, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut debug_list = f.debug_list();
        match &self.0 {
            &[] => {}
            &[entry] => {
                debug_list.entry(entry);
            }
            &[first, second] => {
                debug_list.entry(&first);
                debug_list.entry(&second);
            }
            &[first, .., last] => {
                debug_list.entry(&first);
                debug_list.entry(&"...");
                debug_list.entry(&last);
            }
        }
        debug_list.finish()
    }
}

impl<T: Display> Display for Preview<'_, T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("[")?;
        match self.0 {
            [] => {}
            [entry] => {
                f.write_fmt(format_args!("{entry}"))?;
            }
            [first, second] => {
                f.write_fmt(format_args!("{first}"))?;
                f.write_str(", ")?;
                f.write_fmt(format_args!("{second}"))?;
            }
            [first, .., last] => {
                f.write_fmt(format_args!("{first}"))?;
                f.write_fmt(format_args!(", ..., "))?;
                f.write_fmt(format_args!("{last}"))?;
            }
        }
        f.write_str("]")
    }
}

pub fn format_slice<T, F>(slice: &[T], f: &mut Formatter) -> fmt::Result
where
    T: fmt::Display,
    F: FnMut(&T, &mut Formatter) -> fmt::Result + Clone,
{
    f.write_str("[")?;
    match slice {
        [] => {}
        [first] => write!(f, "{first}")?,
        [first, .., last] => write!(f, "{first}, .., {last}")?,
    }
    f.write_str("]")
}

/// Formats the contents of a list of items, using an ellipsis to indicate when
/// the `length` of the list is greater than `limit`.
///
/// # Parameters
///
/// * `f`: The formatter.
/// * `length`: The length of the list.
/// * `limit`: The maximum number of items before overflow.
/// * `separator`: Separator to write between items.
/// * `ellipsis`: Ellipsis for indicating overflow.
/// * `fmt_elem`: A function that formats an element in the list, given the
///   formatter and the index of the item in the list.
fn format_with_overflow(
    f: &mut Formatter,
    length: usize,
    limit: usize,
    separator: &str,
    ellipsis: &str,
    fmt_elem: &mut dyn FnMut(&mut Formatter, usize) -> fmt::Result,
) -> fmt::Result {
    if length == 0 {
    } else if length <= limit {
        fmt_elem(f, 0)?;
        for i in 1..length {
            f.write_str(separator)?;
            fmt_elem(f, i)?
        }
    } else {
        let edge = limit / 2;
        fmt_elem(f, 0)?;
        for i in 1..edge {
            f.write_str(separator)?;
            fmt_elem(f, i)?;
        }
        f.write_str(separator)?;
        f.write_str(ellipsis)?;
        for i in length - edge..length {
            f.write_str(separator)?;
            fmt_elem(f, i)?
        }
    }
    Ok(())
}
