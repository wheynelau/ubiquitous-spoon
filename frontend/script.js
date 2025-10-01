// Configuration
const CONFIG = {
    BASE_URL: 'http://localhost:3000', // Default backend URL
    ENDPOINTS: {
        SHORTEN: '/shorten'
    }
};

// DOM elements
const elements = {
    form: document.getElementById('urlForm'),
    urlInput: document.getElementById('urlInput'),
    expirationInput: document.getElementById('expirationInput'),
    submitBtn: document.getElementById('submitBtn'),
    btnText: document.querySelector('.btn-text'),
    btnLoading: document.querySelector('.btn-loading'),
    errorMessage: document.getElementById('errorMessage'),
    resultContainer: document.getElementById('resultContainer'),
    shortUrl: document.getElementById('shortUrl'),
    copyBtn: document.getElementById('copyBtn')
};

// State management
let isLoading = false;

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    setupEventListeners();
    // Try to get backend URL from environment or use default
    const backendUrl = getBackendUrl();
    if (backendUrl) {
        CONFIG.BASE_URL = backendUrl;
    }
    
    // Check if this is a shortened URL redirect
    handleShortUrlRedirect();
});

// Setup event listeners
function setupEventListeners() {
    elements.form.addEventListener('submit', handleFormSubmit);
    elements.copyBtn.addEventListener('click', handleCopyClick);
    
    // Clear error when user starts typing
    elements.urlInput.addEventListener('input', clearError);
    elements.expirationInput.addEventListener('input', clearError);
    
    // Handle Enter key in input
    elements.urlInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter' && !isLoading) {
            handleFormSubmit(e);
        }
    });
    
    elements.expirationInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter' && !isLoading) {
            handleFormSubmit(e);
        }
    });
}

// Handle form submission
async function handleFormSubmit(e) {
    e.preventDefault();
    
    if (isLoading) return;
    
    const url = elements.urlInput.value.trim();
    const expirationDays = elements.expirationInput.value.trim();
    
    // Validate URL
    if (!url) {
        showError('Please enter a URL');
        return;
    }
    
    if (!isValidUrl(url)) {
        showError('Please enter a valid URL (e.g., https://example.com)');
        return;
    }
    
    // Validate expiration days if provided
    if (expirationDays) {
        const days = parseInt(expirationDays, 10);
        if (isNaN(days) || days < 1 || days > 365) {
            showError('Expiration days must be between 1 and 365');
            return;
        }
    }
    
    await shortenUrl(url, expirationDays);
}

// Shorten URL function
async function shortenUrl(url, expirationDays) {
    setLoading(true);
    clearError();
    hideResult();
    
    try {
        // Build request body
        const requestBody = { url: url };
        if (expirationDays) {
            requestBody.expiration_days = parseInt(expirationDays, 10);
        }
        
        const response = await fetch(`${CONFIG.BASE_URL}${CONFIG.ENDPOINTS.SHORTEN}`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(requestBody)
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`Server error: ${response.status} - ${errorText}`);
        }
        
        const data = await response.json();
        
        if (data.short_code) {
            showResult(data.short_code);
        } else {
            throw new Error('Invalid response from server');
        }
        
    } catch (error) {
        console.error('Error shortening URL:', error);
        showError(`Failed to shorten URL: ${error.message}`);
    } finally {
        setLoading(false);
    }
}

// Handle copy to clipboard
async function handleCopyClick() {
    const url = elements.shortUrl.value;
    
    if (!url) return;
    
    try {
        await navigator.clipboard.writeText(url);
        
        // Visual feedback
        const originalText = elements.copyBtn.textContent;
        elements.copyBtn.textContent = '✅ Copied!';
        elements.copyBtn.style.background = '#28a745';
        
        setTimeout(() => {
            elements.copyBtn.textContent = originalText;
            elements.copyBtn.style.background = '#28a745';
        }, 2000);
        
    } catch (error) {
        console.error('Failed to copy:', error);
        
        // Fallback: select text for manual copy
        elements.shortUrl.select();
        elements.shortUrl.setSelectionRange(0, 99999);
        
        showError('Copy failed. Text is selected - press Ctrl+C to copy manually.');
    }
}

// Utility functions
function setLoading(loading) {
    isLoading = loading;
    
    if (loading) {
        elements.submitBtn.disabled = true;
        elements.btnText.style.display = 'none';
        elements.btnLoading.style.display = 'inline';
    } else {
        elements.submitBtn.disabled = false;
        elements.btnText.style.display = 'inline';
        elements.btnLoading.style.display = 'none';
    }
}

function showError(message) {
    elements.errorMessage.textContent = message;
    elements.errorMessage.style.display = 'block';
    
    // Auto-hide error after 5 seconds
    setTimeout(() => {
        clearError();
    }, 5000);
}

function clearError() {
    elements.errorMessage.style.display = 'none';
    elements.errorMessage.textContent = '';
}

function showResult(shortUrl) {
    elements.shortUrl.value = shortUrl;
    elements.resultContainer.style.display = 'block';
    
    // Scroll to result
    elements.resultContainer.scrollIntoView({ 
        behavior: 'smooth', 
        block: 'nearest' 
    });
}

function hideResult() {
    elements.resultContainer.style.display = 'none';
}

function isValidUrl(string) {
    try {
        const url = new URL(string);
        return url.protocol === 'http:' || url.protocol === 'https:';
    } catch (_) {
        return false;
    }
}

function getBackendUrl() {
    // Try to get from environment variable or meta tag
    const metaTag = document.querySelector('meta[name="base-url"]');
    if (metaTag) {
        const baseUrl = metaTag.getAttribute('content');
        // Check if the substitution worked (not the literal placeholder)
        if (baseUrl && baseUrl !== '${BASE_URL}') {
            return baseUrl;
        }
    }
    
    // For localhost (both development and Docker), use localhost:3000
    if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
        return 'http://localhost:3000';
    }
    
    // For production, you might want to use the same host
    return window.location.origin.replace(/:\d+$/, ':3000');
}

// Handle shortened URL redirects
async function handleShortUrlRedirect() {
    const path = window.location.pathname;
    
    // Skip if it's the root path or has a file extension
    if (path === '/' || path.includes('.')) {
        return;
    }
    
    // Extract short code from path (remove leading slash)
    const shortCode = path.substring(1);
    
    // Only process if it looks like a short code (alphanumeric, 6+ chars)
    if (!/^[a-zA-Z0-9]{6,}$/.test(shortCode)) {
        return;
    }
    
    try {
        // Show loading state
        document.body.innerHTML = `
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh; font-family: Arial, sans-serif;">
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 1rem;">🔗</div>
                    <div style="font-size: 1.2rem; color: #666;">Redirecting...</div>
                </div>
            </div>
        `;
        
        // Get the original URL from backend
        const response = await fetch(`${CONFIG.BASE_URL}/${shortCode}`, {
            method: 'GET',
            redirect: 'manual' // Don't follow redirects automatically
        });
        
        if (response.status === 302 || response.status === 301) {
            // Get the redirect URL from Location header
            const redirectUrl = response.headers.get('Location');
            if (redirectUrl) {
                window.location.href = redirectUrl;
                return;
            }
        }
        
        // If no redirect, show error
        throw new Error('Short URL not found');
        
    } catch (error) {
        console.error('Redirect error:', error);
        // Show error page
        document.body.innerHTML = `
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh; font-family: Arial, sans-serif;">
                <div style="text-align: center;">
                    <div style="font-size: 2rem; margin-bottom: 1rem;">❌</div>
                    <div style="font-size: 1.2rem; color: #d32f2f; margin-bottom: 1rem;">Short URL not found</div>
                    <a href="/" style="color: #1976d2; text-decoration: none;">← Back to URL Shortener</a>
                </div>
            </div>
        `;
    }
}

// Export for testing (if needed)
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        isValidUrl,
        shortenUrl,
        CONFIG
    };
}
