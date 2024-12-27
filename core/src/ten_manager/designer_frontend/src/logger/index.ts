import pino from "pino";

export const pinoLogger = pino({
  transport: {
    target: "pino-pretty",
    options: {
      colorize: true,
      translateTime: "SYS:standard",
      ignore: "hostname,pid",
    },
  },
  browser: {
    serialize: true,
    asObject: true,
    formatters: {
      level: (label) => ({ level: label }),
      log: ({ time, ...object }) => ({
        time: new Date(time as string).toLocaleString(),
        ...object,
      }),
    },
  },
});

export default pinoLogger;
