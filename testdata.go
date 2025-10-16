package paper

import (
	"encoding/json"
	"fmt"
	"os"
)

const testDataPath = "testdata/files.json"

type TestFile struct {
	Path         string                 `json:"path"`
	Producer     string                 `json:"producer"`
	Pages        int                    `json:"pages"`
	CreationDate string                 `json:"creation_date"`
	Encrypted    bool                   `json:"encrypted"`
	Images       int                    `json:"images"`
	Forms        int                    `json:"forms"`
	Annotations  map[string]interface{} `json:"annotations"`
}

type TestData struct {
	Data []TestFile `json:"data"`
}

func ReadTestData() (*TestData, error) {
	file, err := os.ReadFile(testDataPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read test data: %v", err)
	}

	var testData TestData
	marshalErr := json.Unmarshal(file, &testData)
	if marshalErr != nil {
		return nil, fmt.Errorf("failed to unmarshal test data: %v", marshalErr)
	}

	return &testData, nil
}
