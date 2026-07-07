// RustPress 搜索功能 - 无 jQuery 依赖版
(function () {
  'use strict';

  var searchData = [];
  var searchIndex = null;
  var isLoaded = false;

  var INPUT_ID = 'nav-search-input';
  var SUGGESTIONS_ID = 'search-suggestions';

  function loadSearchData() {
    return fetch('/search.json', { cache: 'no-store' })
      .then(function (res) { return res.ok ? res.json() : []; })
      .then(function (data) {
        searchData = Array.isArray(data) ? data : (data.items || []);
        searchData = searchData.map(function (item, idx) {
          return {
            id: idx,
            title: item.title || item.slug || '',
            content: item.content || '',
            url: item.url || '',
            date: item.date || '',
            tags: item.tags || [],
            categories: item.categories || []
          };
        });
        // 构建 Lunr 索引
        if (typeof lunr !== 'undefined') {
          searchIndex = lunr(function () {
            this.ref('id');
            this.field('title', { boost: 10 });
            this.field('content', { boost: 5 });
            this.field('tags', { boost: 8 });
            this.field('categories', { boost: 6 });
            this.pipeline.remove(lunr.stemmer);
            this.searchPipeline.remove(lunr.stemmer);
            for (var i = 0; i < searchData.length; i++) {
              this.add(searchData[i]);
            }
          });
        }
        isLoaded = true;
      })
      .catch(function () { isLoaded = true; });
  }

  // 生成摘要
  function excerpt(text, query, maxLen) {
    maxLen = maxLen || 150;
    var lower = text.toLowerCase();
    var idx = lower.indexOf(query.toLowerCase());
    if (idx === -1) return text.slice(0, maxLen) + (text.length > maxLen ? '...' : '');
    var start = Math.max(0, idx - 50);
    var end = Math.min(text.length, idx + query.length + 100);
    var out = text.slice(start, end);
    if (start > 0) out = '...' + out;
    if (end < text.length) out = out + '...';
    return out;
  }

  function highlight(text, query) {
    if (!query) return text;
    var safe = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    return text.replace(new RegExp('(' + safe + ')', 'gi'), '<mark class="bg-yellow-200 px-0.5">$1</mark>');
  }

  function performSearch(query) {
    var el = document.getElementById(SUGGESTIONS_ID);
    if (!el) return;

    // 显示搜索结果时隐藏导航快捷键
    var shortcuts = document.getElementById('search-shortcuts');
    if (shortcuts) shortcuts.classList.add('hidden');

    if (!query.trim()) { el.innerHTML = ''; el.classList.add('hidden'); return; }
    if (!isLoaded) { el.innerHTML = '<div class="p-3 text-sm text-mist-green">索引加载中…</div>'; el.classList.remove('hidden'); return; }

    var results = [];
    if (searchIndex) {
      try {
        results = searchIndex.search(query);
      } catch (_) { results = []; }
    }

    // 降级文本匹配
    if (results.length === 0) {
      var qLower = query.toLowerCase();
      searchData.forEach(function (item) {
        var text = (item.title + ' ' + item.content + ' ' + item.tags.join(' ') + ' ' + item.categories.join(' ')).toLowerCase();
        if (text.indexOf(qLower) !== -1) {
          results.push({ ref: item.id, score: 0 });
        }
      });
    }

    // 最多显示 8 条
    results = results.slice(0, 8);

    if (results.length === 0) {
      el.innerHTML = '<div class="p-3 text-sm text-mist-green">未找到相关内容</div>';
      el.classList.remove('hidden');
      return;
    }

    var html = '';
    results.forEach(function (r) {
      var item = searchData[r.ref] || searchData[parseInt(r.ref)];
      if (!item) return;
      var title = highlight(item.title, query);
      var desc = highlight(excerpt(item.content, query, 120), query);
      html += '<a href="' + item.url + '" class="block px-4 py-2.5 text-sm hover:bg-air-bg border-b border-sage-line/10 last:border-0 transition-colors">';
      html += '<div class="font-medium text-moss-ink">' + title + '</div>';
      html += '<div class="text-mist-green text-xs mt-0.5 line-clamp-2">' + desc + '</div>';
      html += '</a>';
    });

    el.innerHTML = html;
    el.classList.remove('hidden');
  }

  // 初始化
  function init() {
    loadSearchData();

    var input = document.getElementById(INPUT_ID);
    if (!input) return;

    var timer = null;
    input.addEventListener('input', function () {
      clearTimeout(timer);
      timer = setTimeout(function () { performSearch(input.value); }, 200);
    });

    input.addEventListener('keydown', function (e) {
      if (e.key === 'Escape') {
        var el = document.getElementById(SUGGESTIONS_ID);
        if (el) el.classList.add('hidden');
        input.blur();
      }
      if (e.key === 'Enter') {
        var el = document.getElementById(SUGGESTIONS_ID);
        if (el) {
          var first = el.querySelector('a');
          if (first) { window.location.href = first.getAttribute('href'); return; }
        }
        // 回车直达搜索页
        if (input.value.trim()) {
          window.location.href = '/search.html?q=' + encodeURIComponent(input.value.trim());
        }
      }
    });

    // 点击外部关闭
    document.addEventListener('click', function (e) {
      var el = document.getElementById(SUGGESTIONS_ID);
      if (el && !e.target.closest('#' + INPUT_ID) && !e.target.closest('#' + SUGGESTIONS_ID)) {
        el.classList.add('hidden');
      }
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
