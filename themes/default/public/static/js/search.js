// RustPress 搜索功能
class RustPressSearch {
    constructor() {
        this.searchData = [];
        this.searchIndex = null;
        this.isLoaded = false;
        this.init();
    }

    async init() {
        await this.loadSearchData();
        this.setupEventListeners();
    }

    async loadSearchData() {
        try {
            const response = await fetch('/search.json');
            this.searchData = await response.json();
            
            // 创建Lunr搜索索引
            const searchData = this.searchData; // 保存引用
            this.searchIndex = lunr(function () {
                this.ref('id');
                this.field('title', { boost: 10 });
                this.field('content', { boost: 5 });
                this.field('tags', { boost: 8 });
                this.field('categories', { boost: 6 });
                
                // 支持中文搜索
                this.pipeline.remove(lunr.stemmer);
                this.searchPipeline.remove(lunr.stemmer);
                
                // 添加文档到索引
                for (let doc of searchData) {
                    this.add(doc);
                }
            });
            
            this.isLoaded = true;
            console.log('搜索索引加载完成，共', this.searchData.length, '篇文章');
        } catch (error) {
            console.error('搜索数据加载失败:', error);
        }
    }

    setupEventListeners() {
        // 点击搜索框或搜索按钮打开弹窗
        $('.search-input').on('click', (e) => {
            e.preventDefault();
            this.openSearchModal();
        });

        $('.search-btn').on('click', (e) => {
            e.preventDefault();
            this.openSearchModal();
        });

        // 弹窗内的搜索输入
        $('#search-modal-input').on({
            input: (e) => {
                this.performSearch($(e.target).val());
            },
            keydown: (e) => {
                if (e.key === 'Escape') {
                    this.closeSearchModal();
                } else if (e.key === 'ArrowDown') {
                    e.preventDefault();
                    this.navigateResults('down');
                } else if (e.key === 'ArrowUp') {
                    e.preventDefault();
                    this.navigateResults('up');
                } else if (e.key === 'Enter') {
                    e.preventDefault();
                    this.selectResult();
                }
            }
        });

        // 关闭弹窗
        $('.search-modal-close').on('click', () => {
            this.closeSearchModal();
        });

        $('.search-modal-overlay').on('click', () => {
            this.closeSearchModal();
        });

        // 键盘快捷键 Ctrl+K 或 Cmd+K
        $(document).on('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                this.openSearchModal();
            }
        });
    }

    openSearchModal() {
        const $searchModal = $('#search-modal');
        const $searchModalInput = $('#search-modal-input');
        
        if ($searchModal.length) {
            $searchModal.css('display', 'flex');
            $('body').css('overflow', 'hidden');
            
            // 聚焦到搜索输入框
            setTimeout(() => {
                if ($searchModalInput.length) {
                    $searchModalInput.focus();
                }
            }, 100);
        }
    }

    closeSearchModal() {
        const $searchModal = $('#search-modal');
        
        if ($searchModal.length) {
            $searchModal.css('display', 'none');
            $('body').css('overflow', '');
            
            this.currentResults = [];
            this.currentIndex = -1;
        }
    }

    performSearch(query) {
        const $searchResults = $('#search-results');
        const $searchCount = $('#search-count');
        
        if (!$searchResults.length) return;

        if (!query.trim()) {
            $searchResults.html('');
            if ($searchCount.length) $searchCount.text('');
            return;
        }

        if (!this.isLoaded) {
            $searchResults.html('<div class="search-loading">搜索索引加载中...</div>');
            return;
        }

        try {
            // 使用Lunr进行搜索
            const results = this.searchIndex.search(query);
            
            // 如果Lunr没有结果，尝试简单的文本匹配
            let finalResults = results;
            if (results.length === 0) {
                finalResults = this.fallbackSearch(query);
            }

            this.displayResults(finalResults, query);
            
            if ($searchCount.length) {
                $searchCount.text(`找到 ${finalResults.length} 个结果`);
            }
        } catch (error) {
            console.error('搜索出错:', error);
            $searchResults.html('<div class="search-error">搜索出错，请重试</div>');
        }
    }

    fallbackSearch(query) {
        const lowerQuery = query.toLowerCase();
        const results = [];
        
        this.searchData.forEach((item, index) => {
            const titleMatch = item.title.toLowerCase().includes(lowerQuery);
            const contentMatch = item.content.toLowerCase().includes(lowerQuery);
            const tagsMatch = item.tags.some(tag => 
                tag.toLowerCase().includes(lowerQuery)
            );
            const categoriesMatch = item.categories.some(cat => 
                cat.toLowerCase().includes(lowerQuery)
            );
            
            if (titleMatch || contentMatch || tagsMatch || categoriesMatch) {
                let score = 0;
                if (titleMatch) score += 10;
                if (contentMatch) score += 5;
                if (tagsMatch) score += 8;
                if (categoriesMatch) score += 6;
                
                results.push({
                    ref: index.toString(),
                    score: score
                });
            }
        });
        
        return results.sort((a, b) => b.score - a.score).slice(0, 10);
    }

    displayResults(results, query) {
        const $searchResults = $('#search-results');
        
        if (results.length === 0) {
            $searchResults.html(`
                <div class="search-no-results">
                    <p>没有找到相关内容</p>
                    <p class="search-suggestion">尝试使用不同的关键词</p>
                </div>
            `);
            return;
        }

        const html = results.map((result, index) => {
            const item = this.searchData[parseInt(result.ref)];
            const excerpt = this.generateExcerpt(item.content, query);
            
            return `
                <div class="search-result-item" data-index="${index}" data-url="${item.url}">
                    <h3 class="search-result-title">
                        <a href="${item.url}">${this.highlightText(item.title, query)}</a>
                    </h3>
                    <p class="search-result-excerpt">${excerpt}</p>
                    <div class="search-result-meta">
                        <span class="search-result-date">${item.date}</span>
                        ${item.tags.length > 0 ? `
                            <span class="search-result-tags">
                                ${item.tags.map(tag => `<span class="tag">${tag}</span>`).join('')}
                            </span>
                        ` : ''}
                    </div>
                </div>
            `;
        }).join('');

        $searchResults.html(html);
        this.selectedIndex = -1;

        // 添加点击事件
        $searchResults.find('.search-result-item').each(function() {
            $(this).on('click', function() {
                const url = $(this).data('url');
                if (url) {
                    window.location.href = url;
                }
            });
        });
    }

    generateExcerpt(content, query, maxLength = 150) {
        const lowerContent = content.toLowerCase();
        const lowerQuery = query.toLowerCase();
        const index = lowerContent.indexOf(lowerQuery);
        
        if (index === -1) {
            return content.substring(0, maxLength) + (content.length > maxLength ? '...' : '');
        }
        
        const start = Math.max(0, index - 50);
        const end = Math.min(content.length, index + query.length + 100);
        let excerpt = content.substring(start, end);
        
        if (start > 0) excerpt = '...' + excerpt;
        if (end < content.length) excerpt = excerpt + '...';
        
        return this.highlightText(excerpt, query);
    }

    highlightText(text, query) {
        if (!query.trim()) return text;
        
        const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
        return text.replace(regex, '<mark>$1</mark>');
    }

    navigateResults(direction) {
        const $items = $('.search-result-item');
        if ($items.length === 0) return;

        // 移除当前选中状态
        if (this.selectedIndex >= 0 && $items[this.selectedIndex]) {
            $($items[this.selectedIndex]).removeClass('selected');
        }

        // 计算新的选中索引
        if (direction === 'down') {
            this.selectedIndex = (this.selectedIndex + 1) % $items.length;
        } else {
            this.selectedIndex = this.selectedIndex <= 0 ? $items.length - 1 : this.selectedIndex - 1;
        }

        // 添加新的选中状态
        if ($items[this.selectedIndex]) {
            const $selectedItem = $($items[this.selectedIndex]);
            $selectedItem.addClass('selected');
            $selectedItem[0].scrollIntoView({ block: 'nearest' });
        }
    }

    selectResult() {
        const $items = $('.search-result-item');
        if (this.selectedIndex >= 0 && $items[this.selectedIndex]) {
            const url = $($items[this.selectedIndex]).data('url');
            if (url) {
                window.location.href = url;
            }
        }
    }
}

// 页面加载完成后初始化搜索
$(document).ready(function() {
    // 检查是否有Lunr.js
    if (typeof lunr === 'undefined') {
        console.warn('Lunr.js 未加载，搜索功能将不可用');
        return;
    }
    
    window.rustpressSearch = new RustPressSearch();
});
