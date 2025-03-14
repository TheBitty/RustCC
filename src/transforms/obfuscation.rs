                    .chars()
                    .map(|c| {
                        (c as u8 ^ key as u8) as char
                    })
                    .collect(); 