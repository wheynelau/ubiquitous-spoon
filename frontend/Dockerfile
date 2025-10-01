# Use nginx to serve static files
FROM nginx:alpine

# Install envsubst for environment variable substitution
RUN apk add --no-cache gettext

# Copy static files to nginx html directory
COPY index.html /usr/share/nginx/html/
COPY styles.css /usr/share/nginx/html/
COPY script.js /usr/share/nginx/html/

# Create a template for index.html with environment variable substitution
RUN cp /usr/share/nginx/html/index.html /usr/share/nginx/html/index.html.template

# Create nginx configuration for SPA
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    \
    # Handle client-side routing \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    \
    # Cache static assets \
    location ~* \.(css|js|png|jpg|jpeg|gif|ico|svg)$ { \
        expires 1y; \
        add_header Cache-Control "public, immutable"; \
    } \
    \
    # Security headers \
    add_header X-Frame-Options "SAMEORIGIN" always; \
    add_header X-Content-Type-Options "nosniff" always; \
    add_header X-XSS-Protection "1; mode=block" always; \
}' > /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Create startup script that substitutes environment variables
RUN echo '#!/bin/sh' > /docker-entrypoint.d/20-envsubst-on-templates.sh && \
    echo 'envsubst '"'"'${BASE_URL}'"'"' < /usr/share/nginx/html/index.html.template > /usr/share/nginx/html/index.html' >> /docker-entrypoint.d/20-envsubst-on-templates.sh && \
    chmod +x /docker-entrypoint.d/20-envsubst-on-templates.sh

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
