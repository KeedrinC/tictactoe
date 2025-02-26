FROM denoland/deno:latest

# Create working directory
WORKDIR /app

# Copy source
COPY . .

# Compile the main app
RUN deno task build
RUN deno cache main.ts --reload

# Run the app
CMD ["run", "-A", "main.ts"]