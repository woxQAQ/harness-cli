class Reader:
    def read(self, key: str) -> str:
        return key.upper()

    def close(self) -> None:
        return None


class CachedReader(Reader):
    def __init__(self) -> None:
        self.cache: dict[str, str] = {}

    def read(self, key: str) -> str:
        if key not in self.cache:
            self.cache[key] = super().read(key)
        return self.cache[key]


class Service:
    class Metrics:
        def __init__(self) -> None:
            self.calls = 0

    def __init__(self, reader: Reader) -> None:
        self.reader = reader
        self.metrics = self.Metrics()

    def run(self, key: str) -> str:
        self.metrics.calls += 1
        return self.reader.read(key)

    def close(self) -> None:
        self.reader.close()


def bootstrap(reader: Reader) -> Service:
    service = Service(reader)
    service.run("startup")
    return service
