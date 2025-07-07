# cookie 🍪


> 🧪 Early Alpha — A work-in-progress with frequent updates and improvements. Your feedback is welcomed!

A lightweight, terminal-based chat client for large language models (LLMs), built in Rust. Chat with OpenAI’s ChatGPT or any provider directly from your terminal.

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

## 🛣️ Roadmap

### 🎯 Milestones

* Chat UI/UX 
  * [x] [Input box] Soft wrap supporting both Vim-style wrapping and Unicode Standard Annex #14 (UAX#14) word boundaries.
    * [ ] Per paragraph cache to improve performance.
  * [x] [Chat messages] Render chat as markdown.
    * [ ] Fix color.
    * [ ] Fix unsupported syntax.
  * [ ] [Chat messages] Make it scrollable.
  * [ ] [Input box] Scrollable buffer, and hard newlines with Shift+Enter.
  * [ ] Cursor Navigation.
  * [ ] [Input box] Input copy paste + cursor better support.
  * [ ] [Chat messages] Tmux like copy mode.
  * [ ] [Input box] Embed nvim.
  * [ ] Vim style nagivation keybindings and help.
* Chat Engine:
  * [x] Retain context across chats.
  * [ ] Web: optional search and crawl.
  * [ ] Track token usage.
* Session Management: persist chat session history and support global search.
* Multi-Provider Support: pluggable provider interface for other LLM backend.
* Configuration & Theming: full config support; custom keymaps.

### ⚠️ Limitations

* Using the [tui-markdown](https://github.com/joshka/tui-markdown) crate for Markdown rendering, which currently supports a subset of markdown features.

## License

Copyright (c) Ashley Zhou <ashleyzhou62@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
