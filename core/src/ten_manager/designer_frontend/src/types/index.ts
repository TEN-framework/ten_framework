export interface SuccessResponse<T> {
  status: "ok";
  data: T;
  meta?: unknown;
}

export interface ErrorResponse {
  status: "fail";
  message: string;
  error?: InnerError;
}

export interface InnerError {
  // Note: The value of 'type' must adhere to the 'error_type' in rust.
  type: string;

  code?: string;
  message: string;
}
