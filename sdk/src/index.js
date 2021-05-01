import Collection from "./collection.js";
import Request from "./request.js";

/**
 * Represents a connection to a JasonDB database.
 * Abstracts over WebSocket.
 */
class JasonDB {
  /**
   * Constructs the JasonDB object instance.
   * This involves connecting to the WebSocket and setting up message handlers.
   * 
   * @param {string} addr - address to connect to, e.g. `localhost`
   * @param {number} port - port to connect to, defaults to 1338
   */
  constructor(addr, port = 1338) {
    this._ws = new WebSocket(`wss://${addr}:${port}`);
    this._ws.onmessage = this._wsRecv.bind(this);
    this._pendingRequests = [];
  }

  /**
   * Requests a collection from the server.
   * If it exists, the promise is resolved with its object, otherwise the promise is rejected.
   * 
   * @param {string} name - name of the collection to return
   * @returns {Promise<Collection>} promise which resolves to the collection object
   */
  collection(name) {
    return this._wsSend(`EXISTS ${name}`)
      .then(exists => {
        if (exists) return Promise.resolve(new Collection(name, this));
        else return Promise.reject("Collection does not exist");
      })
  }

  /**
   * Creates a collection on the server.
   * If a collection already exists, the promise is rejected.
   * 
   * @param {string} name - name of the collection to create
   * @returns {Promise<Collection>} promise which resolves to the created collection object
   */
  create(name) {
    return this._wsSend(`CREATE ${name}`)
      .then(() => Promise.resolve(new Collection(name, this)))
      .catch((err) => Promise.reject(err));
  }

  /**
   * Handles incoming messages from the server.
   * These are reponses to previous requests, so they are matched up using their IDs.
   * 
   * @param {MessageEvent} e - message event from the server
   */
  _wsRecv(e) {
    let splitData = e.data.split(" ");
    let id = splitData[1];
    splitData.splice(0, 2);
    let response = JSON.parse(splitData.join(" "));

    let requestIndex = this._pendingRequests.findIndex(req => req.id == id);
    if (requestIndex !== -1) {
      if (response.status === "success") this._pendingRequests[requestIndex].resolve(response.data);
      else this._pendingRequests[requestIndex].reject(response.message);
      this._pendingRequests.splice(requestIndex, 1);
    }
  }

  /**
   * Handles outgoing messages to the server.
   * Gives them unique timestamp IDs so they can be kept track of.
   * 
   * @param {string} message 
   * @returns {Promise<string>} a promise which resolves when the request is fulfilled
   */
  _wsSend(message) {
    let id = new Date().getTime();
    let messageString = `ID ${id} ${message}`;

    return new Promise((resolve, reject) => {
      let request = new Request(id, resolve, reject);
      this._pendingRequests.push(request);
      this._ws.send(messageString);
    });
  }
}

export default JasonDB;