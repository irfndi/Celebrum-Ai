# Use an official Node.js runtime as a parent image
FROM node:20-slim

# Set the working directory in the container
WORKDIR /app

# Install pnpm
RUN npm install -g pnpm

# Copy package.json and pnpm-lock.yaml to the working directory
COPY package.json pnpm-lock.yaml ./

# Install any needed packages
RUN pnpm install --prod

# Copy the rest of the application's code
COPY . .

# Make port 8787 available to the world outside this container
EXPOSE 8787

# Define environment variable
ENV NODE_ENV production

# Run the app when the container launches
CMD ["pnpm", "start"]