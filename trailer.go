package paper

import "os"

type Trailer struct {
}

func ReadTrailer(file *os.File) (*Trailer, error) {

	trailer := &Trailer{}

	return trailer, nil
}
