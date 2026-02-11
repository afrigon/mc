use toml_edit::Table;

pub fn set_comment(table: &mut Table, key: &str, comments: Vec<&str>) {
    if let Some(mut k) = table.key_mut(key) {
        let mut output = String::new();

        output.push_str("\n");

        for comment in comments {
            output.push_str(format!("# {}\n", comment).as_str());
        }

        k.leaf_decor_mut().set_prefix(output);
    }
}
