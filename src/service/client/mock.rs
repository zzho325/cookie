use async_trait::async_trait;
use color_eyre::Result;

use crate::service::client::{
    OpenAIClient,
    api::{ContentItem, OutputItem, ResponsesReq, ResponsesResp, Role},
};

const MARKDOWN_RESP: &str = "
# Heading Level 1

## Heading Level 2

**Bold Text**, *Italic Text*, ~~Strikethrough~~

`Inline code`

> Blockquote example

1. Numbered list item  
2. Another item

- Bullet list item
- Another bullet

[Link text](https://example.com)

![Alt text](https://example.com/image.png)

```rust
// Fenced code block
fn main() {
    println!(\"Hello, world!\");
}
```

Inline HTML: <span style=\"color:blue\">blue text</span>.

Line one with two spaces at end for  
line break demonstration.

Theme break:

---

- [x] Completed task
- [ ] Pending task

[Link example](https://example.com) and an image: ![Alt text](https://example.com/image.png)

| Col1 | Col2 |
|------|------|
| A    | B    |

This references a footnote[^1].

[^1]: This is the footnote text.

E=mc^2^ and water is H~2~O.
";

/// Mock OpenAI Rest API client.
///
/// It simply echos request message back.
#[cfg(debug_assertions)]
pub struct MockOpenAIClient {}

#[async_trait]
#[cfg(debug_assertions)]
impl OpenAIClient for MockOpenAIClient {
    async fn responses(&self, req: ResponsesReq) -> Result<ResponsesResp> {
        let message = req.input[0].content.clone();
        let resp = match message.as_str() {
            "long" => "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100),
            "markdown" => MARKDOWN_RESP.to_string(),
            _ => message,
        };

        Ok(ResponsesResp {
            id: "mock-response-id".to_string(),
            output: vec![OutputItem::Message {
                role: Role::Assistant,
                content: vec![ContentItem::OutputText { text: resp }],
            }],
        })
    }
}
