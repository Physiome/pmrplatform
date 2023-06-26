use std::cmp::min;

const OTHER_WS: std::ops::RangeInclusive<u8> = 0x9..=0xd;
const BS_ESC_FF: [u8; 3] = [8, 27, 12];
const BYTES_TO_CHECK: usize = 8000;

// roughly implements `git_str_is_binary` from `libgit2`, skipped the
// UTF BOM detection for now...
pub(crate) fn is_binary(data: &[u8]) -> bool {
    match data[..min(data.len(), BYTES_TO_CHECK)]
        .iter()
        .try_fold((0, 0), |(printable, nonprintable), &c| {
            // let result =
            if c == 0 {
                Err(())
            }
            else {
                // as per libgi2/src/util/str.c
                // Printable characters are those above SPACE, excluding
                // DEL, including BS, ESC and FF
                Ok(if (c > 31 && c != 127) || BS_ESC_FF.contains(&c) {
                    (printable + 1, nonprintable)
                }
                else if OTHER_WS.contains(&c) {
                    (printable, nonprintable)
                }
                else {
                    (printable, nonprintable + 1)
                })
            }
            // ;
            // println!("{} {:#04x} - {:?}", c as char, c, result);
            // result
        })
    {
        Err(_) => true,
        Ok((printable, nonprintable)) => {
            (printable >> 7) < nonprintable
        }
    }
}

#[test]
fn test_is_binary() {
    // these are copied over verbatim from libgit2.
    let data0 = b"Simple text\n";
    let data1 = "Is that UTF-8 data I see…\nYep!\n".as_bytes();
    let data2 = b"Internal NUL!!!\0\n\nI see you!\n";
    let data3 = b"\xef\xbb\xbfThis is UTF-8 with a BOM.\n";
    assert!(!is_binary(data0));
    assert!(!is_binary(data1));
    assert!(is_binary(data2));
    assert!(!is_binary(data3));
}
