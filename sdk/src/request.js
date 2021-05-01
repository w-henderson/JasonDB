/**
 * Represents a request.
 * Allows for resolution of promises from different threads.
 */
class Request {
  /**
   * Creates a request.
   * 
   * @param {string} id - the unique ID to keep track of the request
   * @param {(any) => void} resolve - callback to resolve the request
   * @param {(string) => void} reject - callback to reject the request
   */
  constructor(id, resolve, reject) {
    this.id = id;
    this.resolve = resolve;
    this.reject = reject;
  }
}

export default Request;