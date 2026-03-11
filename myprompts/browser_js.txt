#!/usr/bin/env node
/**
 * Browser Automation CLI via Chrome DevTools Protocol (CDP)
 * Commander-based CLI with 12 commands for AI agent control
 * Author: Inspired by Chris's AI Agent Browser Controller
 */

const { program } = require('commander');
const CDP = require('chrome-remote-interface');
const { spawn } = require('child_process');
const fs = require('fs');

const DEFAULT_PORT = 9222;
const DEFAULT_TIMEOUT = 10000;

class BrowserCDP {
  constructor(port = DEFAULT_PORT) {
    this.port = port;
    this.client = null;
  }

  async connect() {
    try {
      this.client = await CDP({ port: this.port });
      const { Network, Page, Runtime, DOM } = this.client;
      await Promise.all([
        Network.enable(),
        Page.enable(),
        DOM.enable(),
        Runtime.enable()
      ]);
      return true;
    } catch (error) {
      throw new Error(`Failed to connect to Chrome on port ${this.port}: ${error.message}`);
    }
  }

  async eval(expression) {
    const { Runtime } = this.client;
    const result = await Runtime.evaluate({
      expression,
      returnByValue: true,
      awaitPromise: true
    });
    return result.result.value;
  }

  async close() {
    if (this.client) {
      await this.client.close();
    }
  }
}

// Command: list - List all open tabs
program
  .command('list')
  .description('List all open Chrome tabs/targets')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (options) => {
    try {
      const targets = await CDP.List({ port: options.port });
      console.log(JSON.stringify(targets, null, 2));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    }
  });

// Command: content - Get page text content
program
  .command('content')
  .description('Get text content from current page')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .option('-s, --selector <string>', 'CSS selector (default: body)')
  .action(async (options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const selector = options.selector || 'body';
      const content = await browser.eval(`
        document.querySelector('${selector}')?.innerText || ''
      `);
      console.log(content);
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: content-all - Get all text content from page
program
  .command('content-all')
  .description('Get all text content from entire page')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const content = await browser.eval('document.body.innerText');
      console.log(content);
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: search - Search for elements on page
program
  .command('search <query>')
  .description('Search for text or elements on page')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (query, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const result = await browser.eval(`
        (() => {
          const elements = Array.from(document.querySelectorAll('*'));
          const matches = elements.filter(el => 
            el.textContent.toLowerCase().includes('${query.toLowerCase()}')
          ).slice(0, 10);
          return matches.map(el => ({
            tag: el.tagName,
            text: el.innerText?.substring(0, 100),
            selector: el.id ? '#' + el.id : el.className ? '.' + el.className.split(' ')[0] : el.tagName
          }));
        })()
      `);
      console.log(JSON.stringify(result, null, 2));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: screenshot - Take screenshot
program
  .command('screenshot [filename]')
  .description('Take screenshot of current page')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (filename, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const { Page } = browser.client;
      const screenshot = await Page.captureScreenshot({ format: 'png' });
      const filepath = filename || `screenshot-${Date.now()}.png`;
      fs.writeFileSync(filepath, screenshot.data, 'base64');
      console.log(JSON.stringify({ success: true, filepath }));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: open - Open/navigate to URL
program
  .command('open <url>')
  .description('Navigate to a URL')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (url, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const { Page } = browser.client;
      await Page.navigate({ url });
      await Page.loadEventFired();
      console.log(JSON.stringify({ success: true, url }));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: click - Click element
program
  .command('click <selector>')
  .description('Click an element by CSS selector')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (selector, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const result = await browser.eval(`
        (() => {
          const el = document.querySelector('${selector}');
          if (!el) return { error: 'Element not found' };
          el.click();
          return { success: true, selector: '${selector}' };
        })()
      `);
      console.log(JSON.stringify(result));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: type - Type into element
program
  .command('type <selector> <text>')
  .description('Type text into an element')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (selector, text, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const result = await browser.eval(`
        (() => {
          const el = document.querySelector('${selector}');
          if (!el) return { error: 'Element not found' };
          el.value = '${text.replace(/'/g, "\'")}';
          el.dispatchEvent(new Event('input', { bubbles: true }));
          el.dispatchEvent(new Event('change', { bubbles: true }));
          return { success: true, text: '${text}' };
        })()
      `);
      console.log(JSON.stringify(result));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: elements - Find elements by selector
program
  .command('elements <selector>')
  .description('Find all elements matching selector')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (selector, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const result = await browser.eval(`
        Array.from(document.querySelectorAll('${selector}')).map(el => ({
          tag: el.tagName,
          id: el.id,
          class: el.className,
          text: el.innerText?.substring(0, 50)
        }))
      `);
      console.log(JSON.stringify(result, null, 2));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: html - Get HTML of element
program
  .command('html [selector]')
  .description('Get HTML of element (default: entire page)')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (selector, options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const html = await browser.eval(
        selector 
          ? `document.querySelector('${selector}')?.innerHTML || ''`
          : 'document.documentElement.outerHTML'
      );
      console.log(html);
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: desc - Describe page structure
program
  .command('desc')
  .description('Describe page structure and key elements')
  .option('-p, --port <number>', 'Chrome debugging port', DEFAULT_PORT)
  .action(async (options) => {
    const browser = new BrowserCDP(options.port);
    try {
      await browser.connect();
      const description = await browser.eval(`
        (() => {
          return {
            title: document.title,
            url: window.location.href,
            buttons: Array.from(document.querySelectorAll('button')).slice(0, 5).map(b => ({
              text: b.innerText?.substring(0, 30),
              id: b.id,
              class: b.className
            })),
            inputs: Array.from(document.querySelectorAll('input, textarea')).slice(0, 5).map(i => ({
              type: i.type,
              placeholder: i.placeholder,
              id: i.id,
              name: i.name
            })),
            links: Array.from(document.querySelectorAll('a')).slice(0, 5).map(a => ({
              text: a.innerText?.substring(0, 30),
              href: a.href
            }))
          };
        })()
      `);
      console.log(JSON.stringify(description, null, 2));
    } catch (error) {
      console.error('Error:', error.message);
      process.exit(1);
    } finally {
      await browser.close();
    }
  });

// Command: close - Close browser connection
program
  .command('close')
  .description('Close CDP connection')
  .action(() => {
    console.log(JSON.stringify({ success: true, message: 'Connection closed' }));
  });

program
  .name('browser')
  .description('Browser automation CLI via Chrome DevTools Protocol')
  .version('1.0.0');

program.parse();
