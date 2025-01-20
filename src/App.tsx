import { createSignal, createEffect, For } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { FaSolidSort } from 'solid-icons/fa';

interface EmojiImage {
  data: string;
  position: number;
}

interface ClipboardContent {
  type: 'Text' | 'Image' | 'RichText';
  content?: string;
  data?: string;
  width?: number;
  height?: number;
  emoji_images?: EmojiImage[];
}

interface ClipboardEntry {
  content: ClipboardContent;
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

  const renderContent = (content: ClipboardContent) => {
    switch (content.type) {
      case 'Text':
        return (
          <div class="content text-content">
            <pre class="text-pre">{content.content}</pre>
          </div>
        );
      case 'RichText':
        return (
          <div class="content rich-text-content">
            {content.content?.split(/(\[微信表情\])/).map((part, index) => {
              if (part === '[微信表情]') {
                const emoji = content.emoji_images?.find(
                  (e) => e.position === content.content?.indexOf(part, index),
                );
                return emoji ? (
                  <img src={emoji.data} alt="emoji" class="inline-emoji" />
                ) : (
                  part
                );
              }
              return <span>{part}</span>;
            })}
          </div>
        );
      case 'Image':
        return (
          <div class="content image-content">
            <img
              src={content.data}
              alt="Clipboard image"
              style={{
                'max-width': '100%',
                'max-height': '300px',
                'object-fit': 'contain',
              }}
            />
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <main class="container">
      <div class="header">
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
                  onClick={() => {
                    if (entry.content.type === 'Text') {
                      navigator.clipboard.writeText(
                        entry.content.content || '',
                      );
                    }
                  }}
                  disabled={entry.content.type === 'Image'}
                >
                  {entry.content.type === 'Text' ? '复制' : '无法复制图片'}
                </button>
              </div>
              {renderContent(entry.content)}
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
