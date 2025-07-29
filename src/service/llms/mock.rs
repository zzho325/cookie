use async_trait::async_trait;
use color_eyre::Result;

use crate::service::llms::{LlmClient, LlmReq, LlmResp};

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
pub struct MockLlmClientImpl {}

#[async_trait]
#[cfg(debug_assertions)]
impl LlmClient for MockLlmClientImpl {
    async fn request(&self, llm_req: LlmReq) -> Result<LlmResp> {
        use crate::models::MessagePayload;

        let msg = if let Some(item) = llm_req.events.last() {
            use crate::models::ChatEventPayload;
            match item {
                ChatEventPayload::Message(payload) => payload.msg.clone(),
                _ => "".to_string(),
            }
        } else {
            "empty history".to_string()
        };
        Ok(LlmResp {
            output: vec![
                MessagePayload {
                    role: crate::models::Role::Assistant,
                    msg,
                }
                .into(),
            ],
            id: "mock-response-id".to_string(),
        })
    }
}
