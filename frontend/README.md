# URL Shortener Frontend

A simple, lightweight frontend for the URL shortener application built with vanilla HTML, CSS, and JavaScript.

## Features

- 🎨 **Modern UI**: Clean, responsive design with gradient background
- ⚡ **Fast**: No framework overhead, pure vanilla JavaScript
- 📱 **Mobile-friendly**: Responsive design that works on all devices
- 🔗 **URL Validation**: Client-side validation for proper URL format
- 📋 **Copy to Clipboard**: One-click copying of shortened URLs
- ⚠️ **Error Handling**: Clear error messages for failed requests
- 🎯 **Loading States**: Visual feedback during API calls

## Files

- `index.html` - Main HTML structure
- `styles.css` - Modern CSS styling with responsive design
- `script.js` - JavaScript functionality for API calls and interactions

## Usage

1. **Local Development**:
   ```bash
   # Serve the files using any web server
   python -m http.server 8000
   # or
   npx serve .
   # or
   # Open index.html directly in browser
   ```

2. **Production Deployment**:
   - Upload all files to your web server
   - Ensure the backend is running on the configured URL
   - No build process required!

## Configuration

The frontend automatically detects the backend URL:
- **Development**: Uses `http://127.0.0.1:3000` for localhost
- **Production**: Uses the same host as the frontend with port 3000

You can also set a custom backend URL by adding a meta tag:
```html
<meta name="backend-url" content="https://your-backend.com">
```

## API Integration

The frontend expects the backend to have:
- `POST /shorten` endpoint that accepts `{"url": "string"}` and returns `{"short_code": "string"}`

## Browser Support

- Modern browsers (Chrome, Firefox, Safari, Edge)
- Requires JavaScript enabled
- Uses modern CSS features (gradients, flexbox, etc.)

## Benefits over Rust/Dioxus

✅ **Simpler**: No compilation, no build process  
✅ **Faster development**: Edit and refresh  
✅ **Smaller**: Just 3 files vs complex Rust setup  
✅ **Universal**: Works anywhere HTML/CSS/JS works  
✅ **Maintainable**: Easy for any developer to understand  
✅ **Deployable**: Upload and go, no Docker needed  

Perfect for simple applications like URL shorteners!
