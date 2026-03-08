#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const args = process.argv.slice(2);
if (args.length === 0) {
    console.log("Usage: claw-fs <COMMAND>");
    process.exit(1);
}

const command = args[0];

function parseArgs() {
    const parsed = {};
    for (let i = 1; i < args.length; i++) {
        if (args[i].startsWith('--')) {
            const key = args[i].substring(2);
            parsed[key] = args[i + 1];
            i++;
        }
    }
    return parsed;
}

function printJsonAndExit(status, message, data = null) {
    const out = { status, message };
    if (data) out.data = data;
    console.log(JSON.stringify(out));
    process.exit(status === 'error' ? 1 : 0);
}

function atomicWrite(filepath, content) {
    const bakPath = filepath + '.bak';
    if (fs.existsSync(filepath)) {
        fs.copyFileSync(filepath, bakPath);
    }
    const tmpPath = filepath + '.tmp';
    fs.writeFileSync(tmpPath, content, 'utf8');
    fs.renameSync(tmpPath, filepath);
}

function readSafe(filepath) {
    if (!fs.existsSync(filepath)) return "";
    return fs.readFileSync(filepath, 'utf8');
}

const p = parseArgs();
const file = p.file;

try {
    if (command === 'append') {
        let text = readSafe(file);
        if (text.length > 0 && !text.endsWith('\n')) text += '\n';
        text += p.content;
        if (!text.endsWith('\n')) text += '\n';
        atomicWrite(file, text);
        const lines = text.split('\n').filter(l => l.length > 0).length;
        const words = text.split(/\s+/).filter(w => w.length > 0).length;
        printJsonAndExit('success', 'Contenido añadido correctamente al final del archivo.', { lines, words });
    }
    else if (command === 'insert') {
        const text = readSafe(file);
        const lines = text.split('\n');
        let idx = parseInt(p.line) - 1;
        if (idx < 0) idx = 0;
        lines.splice(idx, 0, p.content);
        atomicWrite(file, lines.join('\n'));
        printJsonAndExit('success', `Texto insertado en la línea ${p.line}.`);
    }
    else if (command === 'replace-lines') {
        const text = readSafe(file);
        const lines = text.split('\n');
        let start = parseInt(p.start) - 1;
        let end = parseInt(p.end) - 1;
        lines.splice(start, end - start + 1, p.content);
        atomicWrite(file, lines.join('\n'));
        printJsonAndExit('success', `Reemplazadas las líneas ${p.start} a ${p.end}.`);
    }
    else if (command === 'read-chunk') {
        const text = readSafe(file);
        const lines = text.split('\n');
        let start = parseInt(p.start) - 1;
        let end = parseInt(p.end) - 1;
        const chunk = lines.slice(start, end + 1).join('\n');
        printJsonAndExit('success', 'Lectura exitosa.', { chunk });
    }
    else if (command === 'find-refs') {
        const text = readSafe(file);
        const lines = text.split('\n');
        const regex = new RegExp(p.query);
        const context = parseInt(p.context) || 3;
        const matches = [];
        lines.forEach((l, i) => {
            if (regex.test(l)) {
                let s = Math.max(0, i - context);
                let e = Math.min(lines.length - 1, i + context);
                let block = lines.slice(s, e + 1).map((line, idx) => `${s + idx + 1}: ${line}`).join('\n');
                matches.push({ line: i + 1, context_block: block });
            }
        });
        if (matches.length > 0) printJsonAndExit('success', `Se encontraron ${matches.length} coincidencias.`, { matches });
        else printJsonAndExit('error', 'No se encontraron resultados.');
    }
    else if (command === 'undo') {
        const bakPath = file + '.bak';
        if (!fs.existsSync(bakPath)) printJsonAndExit('error', 'No hay backup para deshacer.');
        fs.renameSync(bakPath, file);
        printJsonAndExit('success', 'Último cambio deshecho correctamente.');
    }
    else if (command === 'stats') {
        const text = readSafe(file);
        const stat = fs.statSync(file);
        const lines = text.split('\n').filter(l => l.length > 0).length;
        const words = text.split(/\s+/).filter(w => w.length > 0).length;
        printJsonAndExit('success', 'Estadísticas del archivo.', { lines, words, size_bytes: stat.size });
    }
    else if (command === 'outline') {
        const text = readSafe(file);
        const lines = text.split('\n');
        const re = /^[\s]*(export\s+|async\s+|public\s+|private\s+|protected\s+)?(class|function|def)\s+(\w+)/;
        const reVar = /^[\s]*(export\s+)?(const|let|var)\s+(\w+)\s*=\s*(async\s*)?(function|\(.*?\)\s*=>)/;
        const outline = [];
        lines.forEach((l, i) => {
            if (re.test(l) || reVar.test(l)) {
                outline.push(`${i + 1}: ${l.trim()}`);
            }
        });
        printJsonAndExit('success', 'Outline extraído con éxito.', { outline });
    }
} catch (err) {
    printJsonAndExit('error', err.message);
}
