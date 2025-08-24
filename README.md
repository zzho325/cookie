# cookie 🍪


> 🧪 Early Alpha — A work-in-progress with frequent updates and improvements. Your feedback is welcomed!

A lightweight, terminal-based chat client for LLMs, built in Rust. Chat with OpenAI’s ChatGPT or any provider directly from your terminal.

<img width="800" alt="Snapshot" src="https://github.com/user-attachments/assets/6135bda3-685b-4d40-8e2a-963da1402775" />

## 🛠️ Getting Started

### Installation

```sh
git clone https://github.com/zzho325/cookie.git
cd cookie
```

### Configuration

```sh
export OPENAI_API_KEY=your_key_here
```

### Usage
```sh
cargo build --release
./target/release/cookie
```

* Type your prompt, `Enter` to send.
* `i` / `Esc` to toggle input mode, `q` to quit. 
* `e` to toggle side bar, `j` / `k` or `Down` / `Up` to navigate sessions.
* `s` to open model selection, `j` / `k` or `Down` / `Up` to select, `Esc` / `Enter` to cancel or save. 
* `Tab` to shift focus.
* `n` to start new session.

## 🛣️ Roadmap

### 🎯 Milestones

* Chat UI/UX 
  * [x] [Input box] Soft wrap.
  * [x] [Input box] Cursor nagivation.
  * [x] [Input box] Scrollable buffer.
  * [x] [Chat messages] Render chat as markdown.
    * [ ] Fix color.
    * [ ] Fix unsupported syntax.
  * [x] [Chat messages] Scroll.
  * [ ] [Chat messages] Cursor navigation.
  * [ ] Select range and copy.
  * [ ] Mouse event.
  * [ ] [Input box] Embed nvim.
* App:
  * [x] Indicate current focused widget.
  * [ ] Help and Keymaps.
  * [x] Load config properly.
  * [ ] UI to update settings.
  * [x] Error popup.
* Chat Engine:
  * [x] Retain context across chats.
  * [x] Web: optional search and crawl.
    * [ ] Support other web search provider.
  * [x] Model selection.
  * [ ] Other LLM providers and provider selection.
  * [x] Streaming.
  * [ ] Track token usage.
* Session Management: 
  * [x] Sessions.
  * [x] Persist sessions to db.
  * [ ] Global search.

### ⚠️ Limitations

* Using the [tui-markdown](https://github.com/joshka/tui-markdown) crate for Markdown rendering, which currently supports a subset of markdown features.

## License

Copyright (c) Ashley Zhou <ashleyzhou62@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
