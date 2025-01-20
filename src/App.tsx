import { createSignal, createEffect, For } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { FaSolidSort } from 'solid-icons/fa';

interface ClipboardEntry {
  content: string;
  timestamp: string;
}

function App() {
  const [clipboardHistory, setClipboardHistory] = createSignal<
    ClipboardEntry[]
  >([]);
  const [isDescending, setIsDescending] = createSignal(true); // 默认倒序（最新的在上面）
  const [searchQuery, setSearchQuery] = createSignal(''); // 新增
  const [isSearching, setIsSearching] = createSignal(false); // 新增

  // 修改获取历史记录的逻辑
  createEffect(() => {
    const intervalId = setInterval(async () => {
      // 只有在不搜索时才自动刷新
      if (!isSearching()) {
        const history = await invoke<ClipboardEntry[]>('get_clipboard_history');
        setClipboardHistory(history);
      }
    }, 1000);

    return () => clearInterval(intervalId);
  });

  // 新增搜索处理函数
  const handleSearch = async (value: string) => {
    setSearchQuery(value);
    setIsSearching(!!value);

    const results = await invoke<ClipboardEntry[]>('search_clipboard_history', {
      query: value,
    });
    setClipboardHistory(results);
  };

  const sortedHistory = () => {
    const history = [...clipboardHistory()];
    return isDescending() ? history.reverse() : history;
  };

  const toggleSort = () => {
    setIsDescending(!isDescending());
  };

  return (
    <main class="container">
      <div class="header">
        <h1>剪贴板历史记录</h1>
        <div class="header-controls">
          {/* 新增搜索框 */}
          <input
            type="search"
            class="search-input"
            placeholder="搜索剪贴板内容..."
            value={searchQuery()}
            onInput={(e) => handleSearch(e.currentTarget.value)}
          />
          <button
            class="sort-button"
            onClick={toggleSort}
            title={isDescending() ? '当前最新在上' : '当前最早在上'}
          >
            <FaSolidSort />
            {isDescending() ? '最新在上' : '最早在上'}
          </button>
        </div>
      </div>

      <div class="clipboard-history">
        <For each={sortedHistory()}>
          {(entry) => (
            <div class="clipboard-entry">
              <div class="entry-header">
                <span class="timestamp">{entry.timestamp}</span>
                <button
                  class="copy-button"
                  onClick={() => navigator.clipboard.writeText(entry.content)}
                >
                  复制
                </button>
              </div>
              <div class="content">{entry.content}</div>
            </div>
          )}
        </For>
        {clipboardHistory().length === 0 && (
          <div class="empty-state">暂无剪贴板记录</div>
        )}
      </div>
    </main>
  );
}

export default App;
