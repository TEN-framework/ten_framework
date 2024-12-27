import z from "zod";

export const RESPONSE_COMMON_SCHEMA = z.object({
  status: z.string(),
  data: z.any(),
});

export const genResSchema = <T>(schema: z.ZodType<T>) => {
  return RESPONSE_COMMON_SCHEMA.extend({
    data: schema,
  });
};
