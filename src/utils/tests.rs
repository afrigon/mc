use crate::utils::process::for_each_line;

#[tokio::test]
async fn lines_are_split_and_decoded_leniently() {
    let output: &[u8] = b"first\r\nsec\xffond\n\nlast";
    let mut lines = Vec::new();

    for_each_line(output, |line| lines.push(line.to_string())).await;

    assert_eq!(lines, ["first", "sec\u{fffd}ond", "", "last"]);
}
