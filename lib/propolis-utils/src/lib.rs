pub trait StringExt {
    /// Tries to drop the first line of a string, otherwise just returns the same string
    fn drop_first_line(self) -> String;
    fn shortify(self, prefix_len: usize, suffix_len: usize, delim: &str) -> String;
}

impl StringExt for &str {
    /// ```rust
    /// use propolis_utils::StringExt;
    /// let input = "first";
    /// let output = "first";
    /// assert_eq!(output, input.drop_first_line());
    /// ```
    ///
    /// ```rust
    /// use propolis_utils::StringExt;
    /// let input = "first
    /// second";
    /// let output = "second";
    /// assert_eq!(output, input.drop_first_line());
    /// ```
    fn drop_first_line(self) -> String {
        self.split_once('\n')
            .map(|(_, rest)| rest)
            .unwrap_or(self)
            .into()
    }

    /// ```rust
    /// use propolis_utils::StringExt;
    /// let input = "sk-XYZ-1234";
    /// assert_eq!("sk..1234", input.shortify(2, 4, ".."));
    /// ```
    fn shortify(self, prefix_len: usize, suffix_len: usize, delim: &str) -> String {
        let len = self.chars().count() - suffix_len;
        let prefix = self[0..prefix_len].to_string();
        let mut suffix = self.to_string();
        let _ = suffix.drain(0..len);
        format!("{prefix}{delim}{suffix}")
    }
}

pub mod md {
    use anyhow::anyhow;
    use markdown::Block;

    /// Get markdown codeblock from string
    ///
    /// ```rust
    /// use propolis_utils::md;
    /// let input = "```csv
    /// 1|2|3|4
    /// ```";
    /// let output = "1|2|3|4";
    /// assert_eq!(output, md::parse_codeblock(&input).unwrap());
    /// ```
    pub fn parse_codeblock(data: &str) -> anyhow::Result<String> {
        match markdown::tokenize(data).as_slice() {
            [Block::CodeBlock(_, code), ..] => Ok(code.into()),
            _ => Err(anyhow!("Unable to extract code block: {}", data)),
        }
    }
}

pub mod csv {
    /// Determines the size of the csv row with the most columns
    ///
    /// ```rust
    /// use propolis_utils::csv;
    /// let s = "1|2|3|4\n1|2|3\n";
    /// assert_eq!(4, csv::max_column_count(&s, '|').unwrap());
    /// ```
    pub fn max_column_count(csv_data: &str, delim: char) -> anyhow::Result<usize> {
        let num_delims = csv_data.split('\n').fold(0, |max, row| {
            max.max(row.match_indices(&delim.to_string()).count())
        });
        Ok(num_delims + 1)
    }

    /// Given csv data, make sure all rows have the same number of columns
    /// In particular this will:
    /// - add empty columns to rows that have too few
    /// - remove columns from rows that have too many
    ///
    /// The target column count can be specified or will be set to the row with the most columns
    ///
    /// ```rust
    /// # use propolis_utils::csv;
    /// let s = "1|2|3|4\n1|2|3\n";
    /// assert_eq!("1|2|3|4\n1|2|3|\n", csv::ensure_columns(&s, '|', None).unwrap());
    /// ```
    ///
    /// ```rust
    /// # use propolis_utils::csv;
    /// let s = "1|2|3|4\n1|2|3\n";
    /// assert_eq!("1\n1\n", csv::ensure_columns(&s, '|', Some(1)).unwrap());
    /// ```
    ///
    /// ```rust
    /// # use propolis_utils::csv;
    /// let s = "1|2|3|4\n1|2|3|4\n";
    /// assert_eq!(s, csv::ensure_columns(&s, '|', Some(4)).unwrap());
    /// ```
    pub fn ensure_columns(
        csv_data: &str,
        delim: char,
        target_column_count: Option<usize>,
    ) -> anyhow::Result<String> {
        let target_column_count = match target_column_count {
            Some(target) => target,
            None => max_column_count(csv_data, delim.to_owned())?,
        };
        let delim_s = delim.to_string();
        Ok(csv_data
            .split('\n')
            .map(|s| -> String {
                if s.trim() == "" {
                    return s.into();
                }
                let s = s.to_string();
                let column_count = s.match_indices(delim).count() + 1;
                match column_count.cmp(&target_column_count) {
                    std::cmp::Ordering::Less => {
                        s + delim_s.repeat(target_column_count - column_count).as_str()
                    }
                    std::cmp::Ordering::Greater => s
                        .split(delim)
                        .take(target_column_count)
                        .collect::<Vec<_>>()
                        .join("\n"),
                    std::cmp::Ordering::Equal => s,
                }
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }
}
