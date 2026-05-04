export type EventMap = {
  ready: Date
  error: Error
}

export interface Reader {
  read(key: string): string
  close(): void
}

export enum Status {
  Idle,
  Busy,
  Closed,
}

export class Service {
  static cache = new Map<string, string>()

  constructor(
    private reader: Reader,
    private status: Status = Status.Idle,
  ) {}

  run(key: string) {
    this.status = Status.Busy
    const value = this.reader.read(key)
    Service.cache.set(key, value)
    return value
  }

  close() {
    this.reader.close()
    this.status = Status.Closed
  }

  static from(reader: Reader) {
    return new Service(reader)
  }
}

export function bootstrap(reader: Reader) {
  const service = Service.from(reader)
  service.run("startup")
  return service
}
