# cookie 🍪


> 🧪 Early Alpha — A work-in-progress with frequent updates and improvements. Your feedback is welcomed!

A lightweight, terminal-based chat client for large language models (LLMs), built in Rust. Chat with OpenAI’s ChatGPT or any provider directly from your terminal.

<img width="1000" alt="Snapshot" src="https://github.com/user-attachments/assets/507b6d26-22da-4ef4-a6e0-56edb06448dd" />

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
* `s` to toggle side bar, `j` / `k` or `Down` / `Up` to navigate sessions.
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
  * [ ] Indicate current focused widget.
  * [ ] Help and Keymaps.
  * [ ] Load config properly.
  * [ ] Update settings at run time.
  * [ ] Error popup.
* Chat Engine:
  * [x] Retain context across chats.
  * [ ] Web: optional search and crawl.
  * [ ] Streaming.
  * [ ] Track token usage.
  * [ ] Search.
* Session Management: 
  * [x] sessions.
  * [ ] persist sessions to db.
  * [ ] global search.
* Multi-Provider Support: other LLM backend.

### ⚠️ Limitations

* Using the [tui-markdown](https://github.com/joshka/tui-markdown) crate for Markdown rendering, which currently supports a subset of markdown features.

## License

Copyright (c) Ashley Zhou <ashleyzhou62@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
