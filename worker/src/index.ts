/**
 * Htmlens Cloudflare Worker
 * 
 * This Worker provides an API for extracting semantic knowledge graphs from HTML pages.
 * 
 * Usage:
 *   GET /?url=https://example.com
 *   
 * Once WASM is built, this will use the htmlens WASM module for processing.
 */

export interface Env {
  // Add any environment bindings here
}

// Frontend HTML generator
async function getFrontendHTML(origin: string): Promise<string> {
  return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HTMLens - Semantic Knowledge Graph Extractor</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; padding: 20px; }
        .container { max-width: 1200px; margin: 0 auto; }
        header { text-align: center; color: white; margin-bottom: 40px; }
        h1 { font-size: 3em; margin-bottom: 10px; text-shadow: 2px 2px 4px rgba(0,0,0,0.2); }
        .tagline { font-size: 1.2em; opacity: 0.9; }
        .card { background: white; border-radius: 12px; padding: 30px; box-shadow: 0 10px 30px rgba(0,0,0,0.2); margin-bottom: 20px; }
        .input-section { margin-bottom: 20px; }
        label { display: block; font-weight: 600; margin-bottom: 8px; color: #555; }
        input[type="url"] { width: 100%; padding: 12px 16px; border: 2px solid #e0e0e0; border-radius: 8px; font-size: 16px; transition: border-color 0.3s; }
        input[type="url"]:focus { outline: none; border-color: #667eea; }
        .button-group { display: flex; gap: 12px; margin-top: 20px; }
        button { flex: 1; padding: 14px 24px; font-size: 16px; font-weight: 600; border: none; border-radius: 8px; cursor: pointer; transition: all 0.3s; text-transform: uppercase; letter-spacing: 0.5px; }
        .btn-primary { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; }
        .btn-primary:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4); }
        .btn-secondary { background: #f5f5f5; color: #666; }
        .btn-secondary:hover:not(:disabled) { background: #e0e0e0; }
        button:disabled { opacity: 0.6; cursor: not-allowed; }
        .loading { display: none; text-align: center; padding: 20px; color: #667eea; }
        .loading.active { display: block; }
        .spinner { border: 3px solid #f3f3f3; border-top: 3px solid #667eea; border-radius: 50%; width: 40px; height: 40px; animation: spin 1s linear infinite; margin: 0 auto 10px; }
        @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
        .results { display: none; }
        .results.active { display: block; }
        .tabs { display: flex; gap: 4px; margin-bottom: 20px; border-bottom: 2px solid #e0e0e0; }
        .tab { padding: 12px 24px; background: transparent; border: none; border-bottom: 3px solid transparent; cursor: pointer; font-weight: 600; color: #666; transition: all 0.3s; }
        .tab.active { color: #667eea; border-bottom-color: #667eea; }
        .tab-content { display: none; position: relative; }
        .tab-content.active { display: block; }
        .copy-btn { position: absolute; top: 10px; right: 10px; padding: 8px 16px; background: #667eea; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: 600; transition: all 0.3s; z-index: 10; }
        .copy-btn:hover { background: #5568d3; transform: translateY(-2px); }
        .copy-btn.copied { background: #10b981; }
        .copy-btn.copied::after { content: ' ✓'; }
        .json-output { background: #f8f9fa; border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; overflow-x: auto; max-height: 600px; font-family: 'Courier New', monospace; font-size: 14px; line-height: 1.5; white-space: pre-wrap; word-wrap: break-word; }
        .json-key { color: #0451a5; font-weight: 600; }
        .json-string { color: #a31515; }
        .json-number { color: #098658; }
        .json-boolean { color: #0000ff; }
        .json-null { color: #808080; }
        .json-bracket { color: #000000; font-weight: bold; }
        .markdown-output { background: #f8f9fa; border: 1px solid #e0e0e0; border-radius: 8px; padding: 20px; max-height: 600px; overflow-y: auto; }
        .error { background: #fee; border: 2px solid #fcc; border-radius: 8px; padding: 16px; color: #c33; margin-top: 20px; display: none; }
        .error-title { font-weight: 600; margin-bottom: 8px; }
        .examples { margin-top: 12px; }
        .example-link { display: inline-block; margin: 4px 8px 4px 0; padding: 6px 12px; background: #f0f0f0; border-radius: 6px; color: #667eea; text-decoration: none; font-size: 14px; transition: background 0.3s; }
        .example-link:hover { background: #e0e0e0; }
        footer { text-align: center; color: white; margin-top: 40px; opacity: 0.8; }
        footer a { color: white; text-decoration: underline; }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🔍 HTMLens</h1>
            <p class="tagline">Extract semantic knowledge graphs from any webpage</p>
        </header>
        <div class="card">
            <div class="input-section">
                <label for="urlInput">Enter a URL to analyze:</label>
                <input type="url" id="urlInput" placeholder="https://example.com" value="https://schema.org" />
                <div class="examples">
                    <strong>Try these examples:</strong>
                    <a href="#" class="example-link" data-url="https://schema.org">schema.org</a>
                    <a href="#" class="example-link" data-url="https://json-ld.org">json-ld.org</a>
                    <a href="#" class="example-link" data-url="https://www.imdb.com">IMDB</a>
                    <a href="#" class="example-link" data-url="https://www.nytimes.com">NY Times</a>
                    <a href="#" class="example-link" data-url="https://github.com">GitHub</a>
                </div>
            </div>
            <div class="button-group">
                <button class="btn-primary" id="analyzeBtn">🚀 Analyze</button>
                <button class="btn-secondary" id="clearBtn">🗑️ Clear</button>
            </div>
            <div class="loading" id="loading"><div class="spinner"></div><p>Analyzing webpage...</p></div>
            <div class="results" id="results">
                <div class="tabs">
                    <button class="tab active" data-tab="summary">Summary</button>
                    <button class="tab" data-tab="graph">Graph</button>
                    <button class="tab" data-tab="jsonld">JSON-LD</button>
                    <button class="tab" data-tab="markdown">Markdown</button>
                </div>
                <div class="tab-content active" id="summary-content">
                    <button class="copy-btn" data-copy="summary">📋 Copy</button>
                    <div class="json-output" id="summaryOutput"></div>
                </div>
                <div class="tab-content" id="graph-content">
                    <button class="copy-btn" data-copy="graph">📋 Copy</button>
                    <div class="json-output" id="graphOutput"></div>
                </div>
                <div class="tab-content" id="jsonld-content">
                    <button class="copy-btn" data-copy="jsonld">📋 Copy</button>
                    <div class="json-output" id="jsonldOutput"></div>
                </div>
                <div class="tab-content" id="markdown-content">
                    <button class="copy-btn" data-copy="markdown">📋 Copy</button>
                    <div class="markdown-output" id="markdownOutput"></div>
                </div>
            </div>
            <div class="error" id="error"><div class="error-title">⚠️ Error</div><div id="errorMessage"></div></div>
        </div>
        <footer><p>Powered by <a href="https://github.com/pondevelopment/htmlens" target="_blank">htmlens</a> on Cloudflare Workers</p></footer>
    </div>
    <script>
        const API_BASE = "${origin}";
        const elements = { urlInput: document.getElementById('urlInput'), analyzeBtn: document.getElementById('analyzeBtn'), clearBtn: document.getElementById('clearBtn'), loading: document.getElementById('loading'), results: document.getElementById('results'), error: document.getElementById('error'), errorMessage: document.getElementById('errorMessage'), summaryOutput: document.getElementById('summaryOutput'), graphOutput: document.getElementById('graphOutput'), jsonldOutput: document.getElementById('jsonldOutput'), markdownOutput: document.getElementById('markdownOutput') };
        console.log('[Frontend] Elements initialized:', { urlInput: elements.urlInput, hasValue: elements.urlInput?.value });
        document.querySelectorAll('.tab').forEach(tab => { tab.addEventListener('click', () => { const targetTab = tab.dataset.tab; document.querySelectorAll('.tab').forEach(t => t.classList.remove('active')); document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active')); tab.classList.add('active'); document.getElementById(targetTab + '-content').classList.add('active'); }); });
        document.querySelectorAll('.example-link').forEach(link => { link.addEventListener('click', (e) => { e.preventDefault(); console.log('[Frontend] Example link clicked:', link.dataset.url); elements.urlInput.value = link.dataset.url; console.log('[Frontend] Input value set to:', elements.urlInput.value); analyze(); }); });
        elements.analyzeBtn.addEventListener('click', analyze);
        elements.clearBtn.addEventListener('click', () => { elements.urlInput.value = ''; elements.results.classList.remove('active'); elements.error.style.display = 'none'; });
        elements.urlInput.addEventListener('keypress', (e) => { if (e.key === 'Enter') analyze(); });
        
        let currentData = null;
        document.querySelectorAll('.copy-btn').forEach(btn => { btn.addEventListener('click', async (e) => { const target = e.currentTarget; const copyType = target.getAttribute('data-copy'); let textToCopy = ''; console.log('[Frontend] Copy button clicked:', copyType); if (copyType === 'summary' && currentData) { textToCopy = JSON.stringify({ url: currentData.url, title: currentData.title, nodeCount: currentData.graph?.nodes?.length || 0, edgeCount: currentData.graph?.edges?.length || 0, jsonldBlockCount: currentData.jsonld?.length || 0, htmlLength: currentData.meta?.htmlLength || 0 }, null, 2); } else if (copyType === 'graph' && currentData) { textToCopy = JSON.stringify(currentData.graph || {}, null, 2); } else if (copyType === 'jsonld' && currentData) { textToCopy = JSON.stringify(currentData.jsonld || [], null, 2); } else if (copyType === 'markdown' && currentData) { textToCopy = currentData.markdown || 'No markdown available'; } try { await navigator.clipboard.writeText(textToCopy); console.log('[Frontend] Copied to clipboard:', textToCopy.length, 'characters'); target.classList.add('copied'); target.textContent = '✓ Copied!'; setTimeout(() => { target.classList.remove('copied'); target.textContent = '📋 Copy'; }, 2000); } catch (err) { console.error('[Frontend] Failed to copy:', err); alert('Failed to copy to clipboard. Please try selecting and copying manually.'); } }); });
        async function analyze() { const inputElement = document.getElementById('urlInput'); const url = (inputElement ? inputElement.value : elements.urlInput.value).trim(); console.log('[Frontend] Starting analysis for URL:', url, '| Raw value:', inputElement?.value); if (!url) { console.log('[Frontend] Error: No URL provided | Input element:', inputElement); showError('Please enter a URL'); return; } if (!isValidUrl(url)) { console.log('[Frontend] Error: Invalid URL format'); showError('Please enter a valid URL (must start with http:// or https://)'); return; } elements.error.style.display = 'none'; elements.results.classList.remove('active'); elements.loading.classList.add('active'); elements.analyzeBtn.disabled = true; try { const apiUrl = API_BASE + '/?url=' + encodeURIComponent(url); console.log('[Frontend] Fetching from API:', apiUrl); const response = await fetch(apiUrl); console.log('[Frontend] Response status:', response.status); const data = await response.json(); console.log('[Frontend] Response data:', data); if (!response.ok) throw new Error(data.error || 'Failed to analyze URL'); displayResults(data); } catch (error) { console.error('[Frontend] Error during analysis:', error); showError(error.message || 'An error occurred while analyzing the URL'); } finally { elements.loading.classList.remove('active'); elements.analyzeBtn.disabled = false; } }
        function displayResults(data) { console.log('[Frontend] Displaying results:', data); currentData = data; var businessSummary = '<div style="line-height: 1.8; color: #333;">'; businessSummary += '<h2 style="color: #667eea; margin-bottom: 16px;">📊 Business Summary</h2>'; businessSummary += '<p style="margin-bottom: 12px;"><strong>Page:</strong> <a href="' + escapeHtml(data.url) + '" target="_blank" style="color: #667eea;">' + escapeHtml(data.title || data.url) + '</a></p>'; var jsonld = data.jsonld || []; var types = jsonld.map(function(b) { return b['@type']; }).filter(Boolean); var uniqueTypes = Array.from(new Set(types)); if (uniqueTypes.length > 0) { businessSummary += '<p style="margin-bottom: 12px;"><strong>Content Type:</strong> ' + uniqueTypes.join(', ') + '</p>'; } var products = jsonld.filter(function(b) { return b['@type'] === 'Product'; }); var productGroups = jsonld.filter(function(b) { return b['@type'] === 'ProductGroup'; }); if (productGroups.length > 0) { businessSummary += '<h3 style="color: #764ba2; margin-top: 20px; margin-bottom: 12px;">🛍️ Product Information</h3>'; productGroups.forEach(function(pg) { var variantCount = pg.hasVariant ? (Array.isArray(pg.hasVariant) ? pg.hasVariant.length : 1) : 0; businessSummary += '<p style="margin-bottom: 8px;"><strong>' + escapeHtml(pg.name || 'Product Group') + '</strong><br>'; if (pg.brand && pg.brand.name) businessSummary += 'Brand: ' + escapeHtml(pg.brand.name) + '<br>'; if (variantCount > 0) businessSummary += 'Variants Available: ' + variantCount + '<br>'; if (pg.offers) { var offers = Array.isArray(pg.offers) ? pg.offers : [pg.offers]; var prices = offers.map(function(o) { return o.price; }).filter(Boolean); if (prices.length > 0) { var minPrice = Math.min.apply(Math, prices); var maxPrice = Math.max.apply(Math, prices); var currency = (offers[0] && offers[0].priceCurrency) || '€'; if (minPrice === maxPrice) { businessSummary += 'Price: ' + currency + minPrice.toFixed(2) + '<br>'; } else { businessSummary += 'Price Range: ' + currency + minPrice.toFixed(2) + ' - ' + currency + maxPrice.toFixed(2) + '<br>'; } } var availabilities = offers.map(function(o) { return o.availability; }).filter(Boolean); var inStock = availabilities.filter(function(a) { return a.includes('InStock'); }).length; var outOfStock = availabilities.filter(function(a) { return a.includes('OutOfStock'); }).length; if (inStock > 0 || outOfStock > 0) { businessSummary += 'Availability: '; if (inStock > 0) businessSummary += '✅ ' + inStock + ' in stock '; if (outOfStock > 0) businessSummary += '❌ ' + outOfStock + ' out of stock'; businessSummary += '<br>'; } } businessSummary += '</p>'; }); } else if (products.length > 0) { businessSummary += '<h3 style="color: #764ba2; margin-top: 20px; margin-bottom: 12px;">🛍️ Products Found</h3>'; businessSummary += '<p>' + products.length + ' product(s) detected on this page.</p>'; } var articles = jsonld.filter(function(b) { return b['@type'] === 'Article' || b['@type'] === 'NewsArticle'; }); var organizations = jsonld.filter(function(b) { return b['@type'] === 'Organization'; }); if (articles.length > 0) { businessSummary += '<h3 style="color: #764ba2; margin-top: 20px; margin-bottom: 12px;">📰 Content</h3>'; businessSummary += '<p>' + articles.length + ' article(s) found.</p>'; } if (organizations.length > 0) { businessSummary += '<h3 style="color: #764ba2; margin-top: 20px; margin-bottom: 12px;">🏢 Organization</h3>'; organizations.forEach(function(org) { businessSummary += '<p><strong>' + escapeHtml(org.name || 'Organization') + '</strong><br>'; if (org.description) businessSummary += escapeHtml(org.description) + '<br>'; if (org.url) businessSummary += '<a href="' + escapeHtml(org.url) + '" target="_blank" style="color: #667eea;">' + escapeHtml(org.url) + '</a><br>'; businessSummary += '</p>'; }); } businessSummary += '<h3 style="color: #764ba2; margin-top: 20px; margin-bottom: 12px;">📈 Technical Insights</h3>'; businessSummary += '<p style="margin-bottom: 8px;">'; businessSummary += 'Structured Data Blocks: ' + jsonld.length + '<br>'; businessSummary += 'Page Size: ' + ((data.meta && data.meta.htmlLength) || 0).toLocaleString() + ' characters<br>'; businessSummary += 'SEO Optimization: ' + (jsonld.length > 0 ? '✅ Yes (JSON-LD present)' : '❌ No structured data found'); businessSummary += '</p></div>'; elements.summaryOutput.innerHTML = businessSummary; elements.graphOutput.innerHTML = syntaxHighlightJson(data.graph || {}); elements.jsonldOutput.innerHTML = syntaxHighlightJson(data.jsonld || []); if (data.markdown) { console.log('[Frontend] Markdown length:', data.markdown.length); elements.markdownOutput.innerHTML = '<pre>' + escapeHtml(data.markdown) + '</pre>'; } else { console.log('[Frontend] No markdown content'); elements.markdownOutput.textContent = 'No markdown content available'; } elements.results.classList.add('active'); console.log('[Frontend] Results displayed'); }
        function showError(message) { elements.errorMessage.textContent = message; elements.error.style.display = 'block'; }
        function isValidUrl(string) { try { const url = new URL(string); return url.protocol === 'http:' || url.protocol === 'https:'; } catch (_) { return false; } }
        function escapeHtml(text) { const map = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;' }; return text.replace(/[&<>"']/g, m => map[m]); }
        function syntaxHighlightJson(json) { if (typeof json !== 'string') json = JSON.stringify(json, null, 2); json = json.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'); return json.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function (match) { let cls = 'json-number'; if (/^"/.test(match)) { if (/:$/.test(match)) { cls = 'json-key'; } else { cls = 'json-string'; } } else if (/true|false/.test(match)) { cls = 'json-boolean'; } else if (/null/.test(match)) { cls = 'json-null'; } return '<span class="' + cls + '">' + match + '</span>'; }); }
    </script>
</body>
</html>`;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    console.log('[Worker] Incoming request:', request.method, url.pathname, url.search);
    
    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      console.log('[Worker] Handling CORS preflight');
      return new Response(null, {
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type",
        },
      });
    }

    // Handle root path - serve frontend or API
    if (url.pathname === "/") {
      const targetUrl = url.searchParams.get("url");
      
      // If no URL parameter, serve the frontend HTML
      if (!targetUrl) {
        console.log('[Worker] Serving frontend HTML');
        const html = await getFrontendHTML(url.origin);
        return new Response(html, {
          headers: {
            "Content-Type": "text/html;charset=UTF-8",
            "Access-Control-Allow-Origin": "*",
          },
        });
      }

      // Validate URL
      console.log('[Worker] Received URL parameter:', targetUrl);
      try {
        new URL(targetUrl);
        console.log('[Worker] URL validation passed');
      } catch (e) {
        console.error('[Worker] Invalid URL:', e);
        return new Response(
          JSON.stringify({ error: "Invalid URL provided" }),
          {
            status: 400,
            headers: {
              "Content-Type": "application/json",
              "Access-Control-Allow-Origin": "*",
            },
          }
        );
      }

      // Fetch the HTML content
      console.log('[Worker] Fetching URL:', targetUrl);
      let htmlContent: string = '';
      let fetchError: string | null = null;
      
      try {
        const fetchResponse = await fetch(targetUrl, {
          headers: {
            'User-Agent': 'HTMLens/0.2.0 (Cloudflare Worker; +https://github.com/pondevelopment/htmlens)',
          },
        });
        
        if (!fetchResponse.ok) {
          fetchError = `Failed to fetch URL: ${fetchResponse.status} ${fetchResponse.statusText}`;
          console.error('[Worker]', fetchError);
        } else {
          htmlContent = await fetchResponse.text();
          console.log('[Worker] Fetched HTML, length:', htmlContent.length);
        }
      } catch (error) {
        fetchError = `Network error: ${error instanceof Error ? error.message : String(error)}`;
        console.error('[Worker] Fetch error:', fetchError);
      }

      // Extract JSON-LD blocks from HTML (simple regex-based extraction)
      const jsonldBlocks: any[] = [];
      if (htmlContent) {
        const scriptRegex = /<script[^>]*type=["']application\/ld\+json["'][^>]*>([\s\S]*?)<\/script>/gi;
        let match;
        while ((match = scriptRegex.exec(htmlContent)) !== null) {
          try {
            const jsonData = JSON.parse(match[1]);
            jsonldBlocks.push(jsonData);
            console.log('[Worker] Found JSON-LD block:', jsonData['@type'] || 'unknown type');
          } catch (e) {
            console.warn('[Worker] Failed to parse JSON-LD block');
          }
        }
        console.log('[Worker] Total JSON-LD blocks found:', jsonldBlocks.length);
      }

      // Extract basic metadata
      const titleMatch = htmlContent?.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
      const title = titleMatch ? titleMatch[1].trim() : 'Unknown Title';
      
      const descMatch = htmlContent?.match(/<meta[^>]*name=["']description["'][^>]*content=["']([^"']*)["']/i);
      const description = descMatch ? descMatch[1] : '';

      // Helper function to format products and variants
      function formatProductData(blocks: any[]): string {
        const productGroups = blocks.filter(b => b['@type'] === 'ProductGroup');
        const products = blocks.filter(b => b['@type'] === 'Product');
        let output = '';

        // Format ProductGroups
        productGroups.forEach(pg => {
          output += `## 🛒 ${pg.name || 'Product Group'}\n\n`;
          output += `| Property | Value |\n`;
          output += `|:---------|:------|\n`;
          if (pg.productGroupID) output += `| **ProductGroup ID** | ${pg.productGroupID} |\n`;
          if (pg.brand?.name) output += `| **Brand** | ${pg.brand.name} |\n`;
          if (pg.variesBy) {
            const varies = Array.isArray(pg.variesBy) ? pg.variesBy.join(', ') : pg.variesBy;
            output += `| **Varies By** | ${varies} |\n`;
          }
          if (pg.hasVariant) {
            const variantCount = Array.isArray(pg.hasVariant) ? pg.hasVariant.length : 1;
            output += `| **Total Variants** | ${variantCount} |\n`;
          }
          if (pg.offers) {
            const offers = Array.isArray(pg.offers) ? pg.offers : [pg.offers];
            const prices = offers.map((o: any) => o.price).filter(Boolean);
            if (prices.length > 0) {
              const minPrice = Math.min(...prices);
              const maxPrice = Math.max(...prices);
              const currency = offers[0]?.priceCurrency || '€';
              const priceRange = minPrice === maxPrice 
                ? `${currency}${minPrice.toFixed(2)}`
                : `${currency}${minPrice.toFixed(2)} - ${currency}${maxPrice.toFixed(2)}`;
              output += `| **Price Range** | ${priceRange} |\n`;
            }
            const availabilities = offers.map((o: any) => o.availability).filter(Boolean);
            const inStock = availabilities.filter((a: any) => a?.includes('InStock')).length;
            const outOfStock = availabilities.filter((a: any) => a?.includes('OutOfStock')).length;
            if (inStock || outOfStock) {
              output += `| **Availability** | ✅ ${inStock} InStock / ❌ ${outOfStock} OutOfStock |\n`;
            }
          }
          output += '\n';

          // Format variants if available
          if (pg.hasVariant) {
            const variants = Array.isArray(pg.hasVariant) ? pg.hasVariant : [pg.hasVariant];
            output += `🔹 **Variants**\n\n`;
            
            // Helper to pad strings to exact width
            const pad = (str: string, width: number) => {
              const s = String(str).substring(0, width); // Truncate if too long
              return s + ' '.repeat(Math.max(0, width - s.length));
            };
            
            // Fixed column widths matching CLI output
            const colWidths = {
              sku: 15,
              variant: 15,
              price: 12,
              availability: 14
            };
            
            // Build header with fixed widths
            const varyProps = pg.variesBy ? (Array.isArray(pg.variesBy) ? pg.variesBy : [pg.variesBy]) : [];
            output += `| ${pad('SKU', colWidths.sku)} |`;
            varyProps.forEach((prop: any) => output += ` ${pad(String(prop), colWidths.variant)} |`);
            output += ` ${pad('Price', colWidths.price)} | ${pad('Availability', colWidths.availability)} |\n`;
            
            // Build separator
            output += `| ${'-'.repeat(colWidths.sku)} |`;
            varyProps.forEach(() => output += ` ${'-'.repeat(colWidths.variant)} |`);
            output += ` ${'-'.repeat(colWidths.price)} | ${'-'.repeat(colWidths.availability)} |\n`;

            // Build rows with fixed widths
            variants.forEach((variant: any) => {
              const sku = variant.sku || variant['@id'] || 'N/A';
              output += `| **${pad(String(sku), colWidths.sku - 4)}** |`;
              
              varyProps.forEach((prop: any) => {
                const value = variant[prop] || variant[prop.toLowerCase()] || '-';
                output += ` ${pad(String(value), colWidths.variant)} |`;
              });
              
              const offer = variant.offers ? (Array.isArray(variant.offers) ? variant.offers[0] : variant.offers) : null;
              const currency = offer?.priceCurrency || 'EUR';
              const price = offer?.price ? `${currency}${Number(offer.price).toFixed(2)}` : '-';
              const avail = offer?.availability ? (offer.availability.includes('InStock') ? '✅ InStock' : '❌ OutOfStock') : '-';
              output += ` ${pad(price, colWidths.price)} | ${pad(avail, colWidths.availability)} |\n`;
            });
            output += '\n';
          }
        });

        // Format standalone Products
        if (products.length > 0) {
          output += `📦 **Products Found: ${products.length}**\n\n`;
          products.forEach(product => {
            output += `### ${product.name || 'Unnamed Product'}\n\n`;
            if (product.brand?.name) output += `• **Brand:** ${product.brand.name}\n`;
            if (product.description) output += `• **Description:** ${product.description}\n`;
            if (product.sku) output += `• **SKU:** ${product.sku}\n`;
            if (product.offers) {
              const offer = Array.isArray(product.offers) ? product.offers[0] : product.offers;
              if (offer.price) output += `• **Price:** ${offer.priceCurrency || '€'}${offer.price}\n`;
              if (offer.availability) output += `• **Availability:** ${offer.availability.includes('InStock') ? '✅ In Stock' : '❌ Out of Stock'}\n`;
            }
            output += '\n';
          });
        }

        return output;
      }

      // Build detailed markdown report
      const date = new Date().toISOString().split('T')[0];
      let markdown = `# 🔍 ${title}\n\n`;
      
      if (description) {
        markdown += `> ${description}\n\n`;
      }
      
      markdown += `| Property | Value |\n`;
      markdown += `|:---------|:------|\n`;
      markdown += `| **URL** | ${targetUrl} |\n`;
      markdown += `| **Analyzed** | ${date} |\n`;
      markdown += `| **JSON-LD Blocks** | ${jsonldBlocks.length} |\n`;
      markdown += `| **Page Size** | ${(htmlContent?.length || 0).toLocaleString()} chars |\n\n`;
      markdown += `---\n\n`;
      
      // Check for products/product groups
      const hasProducts = jsonldBlocks.some(b => b['@type'] === 'Product' || b['@type'] === 'ProductGroup');
      
      if (hasProducts) {
        markdown += formatProductData(jsonldBlocks);
      }
      
      // Show all structured data found
      if (jsonldBlocks.length > 0) {
        markdown += `## 📋 All Structured Data (${jsonldBlocks.length} blocks)\n\n`;
        jsonldBlocks.forEach((block, idx) => {
          const type = block['@type'] || 'Unknown';
          const name = block.name || block.headline || block.title || '';
          markdown += `### ${idx + 1}. ${type}${name ? ` - ${name}` : ''}\n\n`;
          
          // Show key properties
          const keyProps = ['name', 'description', 'url', 'image', 'author', 'datePublished', 'dateModified'];
          keyProps.forEach(prop => {
            if (block[prop]) {
              const value = typeof block[prop] === 'object' ? (block[prop].name || block[prop].url || JSON.stringify(block[prop])) : block[prop];
              markdown += `• **${prop}:** ${value}\n`;
            }
          });
          markdown += '\n';
          
          // Collapsible JSON
          markdown += `<details>\n<summary>View Full JSON</summary>\n\n\`\`\`json\n${JSON.stringify(block, null, 2)}\n\`\`\`\n</details>\n\n`;
        });
      } else {
        markdown += `## ℹ️ No Structured Data Found\n\nNo JSON-LD blocks were detected on this page.\n\n`;
      }
      
      markdown += `---\n\n`;
      markdown += `📊 **Stats:** ${jsonldBlocks.length} JSON-LD blocks • ${(htmlContent?.length || 0).toLocaleString()} chars\n\n`;
      markdown += `*Generated by [HTMLens](https://github.com/pondevelopment/htmlens)*`;

      // TODO: Once WASM is built, use it for full processing
      const responseData = {
        url: targetUrl,
        title: title,
        description: description,
        graph: {
          nodes: jsonldBlocks.map((block, idx) => ({
            id: `jsonld-${idx}`,
            type: block['@type'] || 'Unknown',
            label: block.name || block['@type'] || `Block ${idx + 1}`,
          })),
          edges: [],
        },
        jsonld: jsonldBlocks,
        markdown: markdown,
        meta: {
          htmlLength: htmlContent?.length || 0,
          jsonldCount: jsonldBlocks.length,
          wasmStatus: 'pending',
        },
      };
      console.log('[Worker] Response data:', responseData);
      return new Response(
        JSON.stringify(responseData, null, 2),
        {
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
          },
        }
      );
    }

    // Handle health check
    if (url.pathname === "/health") {
      return new Response(
        JSON.stringify({ status: "healthy", timestamp: new Date().toISOString() }),
        {
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
          },
        }
      );
    }

    return new Response("Not Found", { status: 404 });
  },
};
