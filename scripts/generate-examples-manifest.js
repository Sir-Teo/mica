#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const examplesDir = path.join(__dirname, '..', 'examples');
const outputFile = path.join(__dirname, '..', 'docs', 'examples-manifest.json');

console.log('📝 Generating examples manifest...');

try {
    const files = fs.readdirSync(examplesDir)
        .filter(file => file.endsWith('.mica'))
        .sort();

    const examples = files.map(filename => {
        const filePath = path.join(examplesDir, filename);
        const content = fs.readFileSync(filePath, 'utf-8');
        const name = filename.replace('.mica', '');
        
        // Get description from first comment line
        let description = '';
        const lines = content.split('\n');
        const firstLine = lines[0].trim();
        if (firstLine.startsWith('//')) {
            description = firstLine.replace(/^\/\/\s*/, '');
        } else if (firstLine.startsWith('module')) {
            description = `Module: ${firstLine.replace('module ', '')}`;
        }

        // Try to extract a better description from documentation or comments
        for (const line of lines) {
            if (line.trim().startsWith('// ') && line.length > 3) {
                description = line.trim().replace(/^\/\/\s*/, '');
                break;
            }
        }

        return {
            id: name,
            name: filename,
            description: description || `Example: ${name}`,
            code: content,
            lines: lines.length,
            size: content.length
        };
    });

    fs.writeFileSync(outputFile, JSON.stringify(examples, null, 2));

    console.log(`✅ Manifest generated: ${outputFile}`);
    console.log(`📊 Total examples: ${examples.length}`);
    console.log(`📦 Total lines of code: ${examples.reduce((sum, ex) => sum + ex.lines, 0)}`);
} catch (error) {
    console.error('❌ Error generating manifest:', error);
    process.exit(1);
}
