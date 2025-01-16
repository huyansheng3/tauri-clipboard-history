import { createSignal, createEffect, For } from 'solid-js';
import logo from './assets/logo.svg';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

interface ClipboardEntry {
  content: string;
  timestamp: string;
}

function App() {
  const [greetMsg, setGreetMsg] = createSignal('');
  const [name, setName] = createSignal('');
  const [clipboardHistory, setClipboardHistory] = createSignal<
    ClipboardEntry[]
  >([]);

  // 定期获取剪贴板历史
  createEffect(() => {
    const intervalId = setInterval(async () => {
      const history = await invoke<ClipboardEntry[]>('get_clipboard_history');
      setClipboardHistory(history);
    }, 1000);

    return () => clearInterval(intervalId);
  });

  async function greet() {
    setGreetMsg(await invoke('greet', { name: name() }));
  }

  return (
    <main class="container">
      <h1>剪贴板历史记录</h1>

      <div class="clipboard-history">
        <For each={clipboardHistory()}>
          {(entry) => (
            <div class="clipboard-entry">
              <div class="timestamp">{entry.timestamp}</div>
              <div class="content">{entry.content}</div>
            </div>
          )}
        </For>
      </div>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg()}</p>
    </main>
  );
}

export default App;
