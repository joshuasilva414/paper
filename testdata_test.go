package paper

import (
	"encoding/json"
	"os"
	"testing"
)

func TestFilesJsonExists(t *testing.T) {
	_, err := os.Stat(testDataPath)
	if err != nil {
		t.Errorf("TestDataPath not found: %v", err)
	}
}

func TestReadFilesJson(t *testing.T) {
	filesJson, err := os.ReadFile(testDataPath)
	if err != nil {
		t.Errorf("Failed to read files.json: %v", err)
	}

	var testData TestData
	marshalErr := json.Unmarshal(filesJson, &testData)
	if marshalErr != nil {
		t.Errorf("Failed to unmarshal files.json: %v", marshalErr)
	}

	t.Logf("Read %d files from files.json", len(testData.Data))
}

func TestReadFile(t *testing.T) {
	testData, err := ReadTestData()
	if err != nil {
		t.Errorf("Failed to read test data: %v", err)
	}

	t.Logf("Read %d files from files.json", len(testData.Data))
}
