pub struct CsvFixup {}

impl CsvFixup {
    /// Determines the size of the csv row with the most columns
    /// ```rust
    /// use propolis_utils::CsvFixup;
    /// let s = "1|2|3|4\n1|2|3\n";
    /// assert_eq!(4, CsvFixup::max_column_count(&s, '|').unwrap());
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
    /// # use propolis_utils::CsvFixup;
    /// let s = "1|2|3|4\n1|2|3\n";
    /// assert_eq!("1|2|3|4\n1|2|3|\n", CsvFixup::ensure_columns(&s, '|', None).unwrap());
    /// ```
    ///
    /// ```rust
    /// # use propolis_utils::CsvFixup;
    /// let s = "1|2|3|4\n1|2|3\n";
    /// assert_eq!("1\n1\n", CsvFixup::ensure_columns(&s, '|', Some(1)).unwrap());
    /// ```
    ///
    /// ```rust
    /// # use propolis_utils::CsvFixup;
    /// let s = "1|2|3|4\n1|2|3|4\n";
    /// assert_eq!(s, CsvFixup::ensure_columns(&s, '|', Some(4)).unwrap());
    /// ```
    pub fn ensure_columns(
        csv_data: &str,
        delim: char,
        target_column_count: Option<usize>,
    ) -> anyhow::Result<String> {
        let target_column_count = match target_column_count {
            Some(target) => target,
            None => Self::max_column_count(&csv_data, delim.to_owned())?,
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
