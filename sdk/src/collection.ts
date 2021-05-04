import JasonDB from "./index.js";

/**
 * Represents a collection of the database.
 * Should never be constructed manually, instead by the `JasonDB` instance upon calling `collection`.
 */
class Collection {
  public id: string;
  private _database: JasonDB;

  /**
   * Constructs a class instance representing a collection.
   * Does not validate as this is done in the `JasonDB.collection` method.
   * 
   * @param {string} id - the name of the collection on the database
   * @param {JasonDB} database - instance of the database of which this collection is a part
   */
  constructor(id: string, database: JasonDB) {
    this.id = id;
    this._database = database;
  }

  /**
   * Gets a document from the collection.
   * If the document does not exist, the promise is rejected.
   * 
   * @param {string} id - the ID of the document to get
   * @returns {Promise<any>} promise resolving to the document object
   */
  get(id: string): Promise<any> {
    return this._database._wsSend(`GET ${id} FROM ${this.id}`)
      .then(data => Promise.resolve(data))
      .catch(err => Promise.reject(err))
  }

  /**
   * Sets a document in the collection to the given value.
   * If the document already exists, it is silently overwritten.
   * 
   * @param {string} id - the ID of the document to set
   * @param {any} value - the value to set the document to, any data type except JSON
   * @returns {Promise<any>} promise which resolves if the set is successful
   */
  set(id: string, value: any): Promise<any> {
    return this._database._wsSend(`SET ${id} FROM ${this.id} TO ${JSON.stringify(value)}`)
      .then(() => Promise.resolve())
      .catch(err => Promise.reject(err))
  }

  /**
   * Gets all of the documents in the collection.
   * If a condition is specified, only lists documents which meet the condition.
   * 
   * @param {string | undefined} condition - condition string, e.g. `country EQ UK`
   * @returns {Promise<any>} promise which resolves to an object containing all the documents
   */
  list(condition?: string): Promise<any> {
    return this._database._wsSend(!condition ? `LIST ${this.id}` : `LIST ${this.id} WHERE ${condition}`)
      .then((data) => Promise.resolve(data))
      .catch(err => Promise.reject(err))
  }

  /**
   * Deletes the collection or a document within it.
   * If no document ID is specified, the entire collection is deleted.
   * 
   * @param {string | undefined} document - ID of the document to delete
   * @returns {Promise<any>} promise which resolves if the delete is successful
   */
  delete(document?: string): Promise<any> {
    return this._database._wsSend(!document ? `DELETE ${this.id}` : `DELETE ${document} FROM ${this.id}`)
      .then(() => Promise.resolve())
      .catch(err => Promise.reject(err))
  }
}

export default Collection;