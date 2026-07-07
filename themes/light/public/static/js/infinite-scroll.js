(function () {
  'use strict';

  var currentPage = 1;
  var loading = false;
  var allLoaded = false;

  var container = document.getElementById('articles');
  if (!container) return;

  // 从 container 的 data 属性中读取当前页码和总页数
  var totalPages = parseInt(container.getAttribute('data-total-pages'), 10) || 1;
  var currentPage = parseInt(container.getAttribute('data-current-page'), 10) || 1;

  // 倒分页模式：最大页号（如 index3.html）为最新内容（与首页 index.html 相同）。
  // 当在首页或最大页时，向下拉应该依次递减加载 index2.html, index1.html ... 直至第 1 页。
  function getNextUrl() {
    if (allLoaded) return null;
    var nextPage = currentPage - 1;
    if (nextPage < 1) return null;
    return '/index' + nextPage + '.html';
  }

  function extractArticles(html) {
    var parser = new DOMParser();
    var doc = parser.parseFromString(html, 'text/html');
    var articles = doc.getElementById('articles');
    if (!articles) return null;

    // 移除分页控件（如果存在）
    var pagination = articles.querySelector('.flex.justify-center.mt-6, .flex.flex-col.items-center');
    if (pagination) pagination.remove();

    var children = [];
    while (articles.firstChild) {
      children.push(articles.removeChild(articles.firstChild));
    }
    return children;
  }

  function loadNextPage() {
    if (loading || allLoaded) return;
    var url = getNextUrl();
    if (!url) { allLoaded = true; return; }

    loading = true;
    fetch(url, { credentials: 'same-origin' })
      .then(function (res) {
        if (!res.ok) { allLoaded = true; return null; }
        return res.text();
      })
      .then(function (html) {
        if (!html) { allLoaded = true; return; }
        var nodes = extractArticles(html);
        if (!nodes || nodes.length === 0) { allLoaded = true; return; }

        // 追加文章节点
        for (var i = 0; i < nodes.length; i++) {
          container.appendChild(nodes[i]);
        }

        currentPage--;
        history.replaceState(null, '', url);
        loading = false;

        // 如果不足一页则停止
        if (nodes.length < 10) allLoaded = true;
      })
      .catch(function () {
        allLoaded = true;
        loading = false;
      });
  }

  // IntersectionObserver 触发
  var sentinel = document.createElement('div');
  sentinel.className = 'scroll-sentinel';
  sentinel.style.height = '1px';
  container.parentNode.appendChild(sentinel);

  var observer = new IntersectionObserver(function (entries) {
    if (entries[0].isIntersecting) loadNextPage();
  }, { rootMargin: '200px' });
  observer.observe(sentinel);

})();
