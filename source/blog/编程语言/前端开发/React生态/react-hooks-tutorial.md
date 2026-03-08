---
title: "React Hooks完全教程"
createTime: 2024-04-05 16:45:00
tags: ["React", "Hooks", "前端框架"]
---

# React Hooks完全教程

React Hooks是React 16.8引入的新特性，让函数组件也能使用状态和其他React特性。

## 基础Hooks

### useState
管理组件状态的Hook。

```jsx
import React, { useState } from 'react';

function Counter() {
    const [count, setCount] = useState(0);
    
    return (
        <div>
            <p>Count: {count}</p>
            <button onClick={() => setCount(count + 1)}>
                Increment
            </button>
        </div>
    );
}
```

### useEffect
处理副作用的Hook。

```jsx
import React, { useState, useEffect } from 'react';

function UserProfile({ userId }) {
    const [user, setUser] = useState(null);
    
    useEffect(() => {
        fetchUser(userId).then(setUser);
    }, [userId]);
    
    return user ? <div>{user.name}</div> : <div>Loading...</div>;
}
```

### useContext
消费Context的Hook。

```jsx
import React, { useContext } from 'react';

const ThemeContext = React.createContext();

function Button() {
    const theme = useContext(ThemeContext);
    return <button className={theme}>Click me</button>;
}
```

## 高级Hooks

### useReducer
管理复杂状态的Hook。

```jsx
import React, { useReducer } from 'react';

function reducer(state, action) {
    switch (action.type) {
        case 'increment':
            return { count: state.count + 1 };
        case 'decrement':
            return { count: state.count - 1 };
        default:
            throw new Error();
    }
}

function Counter() {
    const [state, dispatch] = useReducer(reducer, { count: 0 });
    
    return (
        <div>
            Count: {state.count}
            <button onClick={() => dispatch({ type: 'increment' })}>+</button>
            <button onClick={() => dispatch({ type: 'decrement' })}>-</button>
        </div>
    );
}
```

### useMemo和useCallback
性能优化的Hook。

## 自定义Hooks

创建可复用的逻辑。

```jsx
function useLocalStorage(key, initialValue) {
    const [storedValue, setStoredValue] = useState(() => {
        try {
            const item = window.localStorage.getItem(key);
            return item ? JSON.parse(item) : initialValue;
        } catch (error) {
            return initialValue;
        }
    });
    
    const setValue = (value) => {
        try {
            setStoredValue(value);
            window.localStorage.setItem(key, JSON.stringify(value));
        } catch (error) {
            console.error(error);
        }
    };
    
    return [storedValue, setValue];
}
```

## 最佳实践

1. 遵循Hooks规则
2. 合理使用依赖数组
3. 避免过度优化
4. 创建自定义Hooks复用逻辑

React Hooks让函数组件变得更加强大和灵活。