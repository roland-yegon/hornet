require.config({ paths: { vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.34.1/min/vs' } });

require(['vs/editor/editor.main'], function () {
    // Define Hornet Language for Monaco
    monaco.languages.register({ id: 'hornet' });

    monaco.languages.setMonarchTokensProvider('hornet', {
        tokenizer: {
            root: [
                [/\b(fn|if|else|for|while|match|import|struct|async|await|return|in)\b/, 'keyword'],
                [/\b\d+\b/, 'number'],
                [/"([^"\\]|\\.)*"/, 'string'],
                [/#.*$/, 'comment'],
                [/[a-zA-Z_][a-zA-Z0-9_]*/, 'identifier'],
            ]
        }
    });

    // Create the Editor
    const editor = monaco.editor.create(document.getElementById('editor-container'), {
        value: [
            'import web',
            '',
            '# Welcome to Hornet Hive!',
            'fn greet(name):',
            '    print("Hello, " + name + "!")',
            '',
            'greet("Hornet Developer")',
            '',
            'for i in 1..5:',
            '    print("Step " + i.str())'
        ].join('\n'),
        language: 'hornet',
        theme: 'vs-dark',
        automaticLayout: true,
        fontFamily: 'JetBrains Mono',
        fontSize: 14,
        lineNumbers: 'on',
        minimap: { enabled: true },
        cursorBlinking: 'smooth',
        smoothScrolling: true,
        contextmenu: true,
        padding: { top: 20 }
    });

    // Handle Run Button
    document.getElementById('run-btn').addEventListener('click', () => {
        const code = editor.getValue();
        const output = document.getElementById('output');
        
        // Simulate compilation and run
        const log = (msg, type) => {
            const div = document.createElement('div');
            div.className = 'line ' + (type || '');
            div.textContent = msg;
            output.appendChild(div);
            output.scrollTop = output.scrollHeight;
        };

        log(`\n> Executing script...`, 'sys');
        setTimeout(() => {
            log('[BUILD] Optimization passes completed.', 'success');
            log('[RUN] Hornet runtime initialized.', 'success');
            log('Hello, Hornet Developer!');
        }, 500);
    });
});
