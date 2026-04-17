FROM node:20-slim AS builder
WORKDIR /build

COPY package*.json ./
RUN npm ci

COPY . .
ARG VITE_EXPLORER_API_URL=http://localhost:8088
ENV VITE_EXPLORER_API_URL=$VITE_EXPLORER_API_URL
RUN npm run build

FROM nginx:alpine
COPY --from=builder /build/dist /usr/share/nginx/html
RUN echo 'server { \
  listen 80; \
  root /usr/share/nginx/html; \
  index index.html; \
  location / { try_files $uri $uri/ /index.html; } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
