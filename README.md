# Paper

A library for reading and writing PDF files.

## Usage

```go

import (
	"github.com/joshuasilva414/paper"
)

pdf, err := paper.Read("test.pdf")
if err != nil {
	fmt.Println(err)
}
```

## License

MIT
