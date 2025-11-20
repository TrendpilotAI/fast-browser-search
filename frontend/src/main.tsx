import React from 'react'
import ReactDOM from 'react-dom/client'

const root = document.getElementById('root');
if (root) {
    ReactDOM.createRoot(root).render(
        <div style={{color: 'green', fontSize: '50px'}}>Minimal Mount</div>
    );
}
