import * as winston from 'winston';
import type { LoggerInterface } from '../types';

const { combine, timestamp, printf, colorize, align, json } = winston.format;

// Custom format for console logging with colors and structure
const consoleFormat = combine(
  colorize({ all: true }),
  timestamp({ format: 'YYYY-MM-DD HH:mm:ss' }),
  align(),
  printf((info) => `[${info.timestamp}] ${info.level}: ${info.message}`)
);

// Custom format for JSON logging (e.g., for sending to a log management system or for structured logs in Cloudflare)
const jsonFormat = combine(
  timestamp(),
  json()
);

export function createLogger(logLevel = 'info'): LoggerInterface {
  const transports = [];

  // In a Cloudflare Worker environment, console.log/error etc. are often the primary way to see logs.
  // Winston can be configured to use these.
  transports.push(
    new winston.transports.Console({
      level: logLevel,
      format: process.env.NODE_ENV === 'production' ? jsonFormat : consoleFormat,
      // handleExceptions: true, // Optional: handle uncaught exceptions
    })
  );

  // If you had other transports, like sending logs to an external service, you'd add them here.
  // e.g., new winston.transports.Http({ ... }) or a custom Cloudflare KV transport.

  const logger = winston.createLogger({
    level: logLevel,
    levels: winston.config.npm.levels, // Use standard npm logging levels (error, warn, info, http, verbose, debug, silly)
    format: combine(
        timestamp({ format: 'YYYY-MM-DD HH:mm:ss.SSS' }),
        printf(info => `[${info.timestamp}] ${info.level.toUpperCase()}: ${info.message}${info.splat !== undefined ? `${info.splat}`: ""}${info.stack !== undefined ? `\n${info.stack}` : ""}`)
    ),
    transports,
    exitOnError: false, // Prevent exit on handled exceptions
  });

  // Winston's logger directly implements methods like info, error, warn, debug.
  // We need to ensure it has a generic 'log' method if LoggerInterface requires it strictly,
  // or adjust LoggerInterface. For now, Winston's logger is largely compatible.
  // If LoggerInterface's `log` method ( (level: string, message: string, ...meta: unknown[]) => void; ) is crucial,
  // you might need to explicitly add it or ensure Winston's built-in `log` matches.
  // Winston's `logger.log({ level: 'info', message: '...' })` is the standard way.

  // To better match LoggerInterface if it's distinct from Winston's native methods:
  const loggerInstance = {
    debug: logger.debug.bind(logger),
    info: logger.info.bind(logger),
    warn: logger.warn.bind(logger),
    error: logger.error.bind(logger),
    // Ensure a generic log method exists if LoggerInterface demands it
    // Winston's logger.log takes an object: logger.log({ level: 'info', message: 'Hello' })
    // Adapting it to (level: string, message: string, ...meta: unknown[]): void
    log: (level, message, ...meta) => {
      logger.log(level, message, ...meta);
    },
  } as LoggerInterface;

  return loggerInstance;
}
