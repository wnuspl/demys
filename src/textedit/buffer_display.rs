use crate::plot::Plot;

pub fn wrap_content(content: String, dim: Plot, mut cursor: usize) -> (Vec<Vec<String>>, Plot) {
    let mut out = Vec::new();

    let mut n = 0;
    let mut cursor_plot = None;
    for line in content.split("\n") {
        let mut subout = Vec::new();

        let mut line = line.to_string();
        loop {
            // take dim.col width of chars into vec
            let partial: String = line
                .drain(..std::cmp::min(line.len(), dim.col))
                .collect();

            // set cursor if on right line
            if cursor_plot.is_none() {
                if cursor <= partial.len() {
                    cursor_plot = Some(Plot::new(n, cursor));
                } else if partial.len() <= cursor {
                    cursor -= partial.len();
                }
            }

            subout.push(partial);

            n += 1;
            // break if hiting end
            if line.len() == 0 || n >= dim.row {
                break;
            }
        }

        if cursor > 0 {
            cursor -= 1;
        }

        out.push(subout);
    }

    if cursor_plot.is_none() {
        cursor_plot = Some(Plot::new(0,0));
    }

    (out, cursor_plot.unwrap())
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wraps_content() {
        let content = "012345\n678910X\nthis line is extraaa long".to_string();

        let dim = Plot::new(10,6);
        let (wrapped, _) = wrap_content(content.clone(), dim, 0);

        assert_eq!(wrapped[0], vec!["012345".to_string()]);
        assert_eq!(wrapped[1], vec!["678910".to_string(), "X".to_string()]);
        assert_eq!(wrapped[2].len(), 5);
    }

    #[test]
    fn test_cursor() {
        let content = "012345\n678910X\nthis line is extraaa long".to_string();

        let dim = Plot::new(10,6);

        let (_, cursor) = wrap_content(content.clone(), dim, 0);
        assert_eq!(cursor, Plot::new(0,0));

        let (_, cursor) = wrap_content(content.clone(), dim, 5);
        assert_eq!(cursor, Plot::new(0,5));

        let (_, cursor) = wrap_content(content.clone(), dim, 6);
        assert_eq!(cursor, Plot::new(0,6));

        let (_, cursor) = wrap_content(content.clone(), dim, 13);
        assert_eq!(cursor, Plot::new(2,0));

        let (_, cursor) = wrap_content(content.clone(), dim, 14);
        assert_eq!(cursor, Plot::new(3,0));
    }

    #[test]
    fn test_cursor_empty_lines() {
        let content = "012345\n\n\nthis line is extraa long".to_string();
        let dim = Plot::new(10,6);

        let (_, cursor) = wrap_content(content.clone(), dim, 0);
        assert_eq!(cursor, Plot::new(0,0));

        let (_, cursor) = wrap_content(content.clone(), dim, 7);
        assert_eq!(cursor, Plot::new(0,6));

        let (_, cursor) = wrap_content(content.clone(), dim, 7);
        assert_eq!(cursor, Plot::new(1,0));

        let (_, cursor) = wrap_content(content.clone(), dim, 8);
        assert_eq!(cursor, Plot::new(2,0));
    }
}