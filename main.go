package main
import (
	"fmt"
	"io"
	"bytes"
    "io/ioutil"
    "mime/multipart"
    "net/http"
    "os"
    "path/filepath"
    // "strconv"
    "strings"
	"encoding/json"
	"flag"
	"crypto/sha1"
	"net/url"
)


type InitResponse struct {
	Code       int    `json:"code"`
	Message    string `json:"message"`
	Submessage string `json:"submessage"`
	Data       struct {
		UploadEp      string `json:"uploadEp"`
		FileName      string `json:"fileName"`
		FileSize      int    `json:"fileSize"`
		ChunkSize     int    `json:"chunkSize"`
		UploadChunks  []any  `json:"uploadChunks"`
		FileCid       string `json:"fileCid"`
		ThumbnailCid  string `json:"thumbnailCid"`
		CoverCid      string `json:"coverCid"`
		UploadState   int    `json:"uploadState"`
		FileMimeType  string `json:"fileMimeType"`
		FileExtension string `json:"fileExtension"`
		FileHash      string `json:"fileHash"`
		QueueExpireTs int    `json:"queueExpireTs"`
	} `json:"data"`
	Rsptime int `json:"rsptime"`
}

func main() {
	token := flag.String("token", "", "登陆token")
	file_path := flag.String("path", "", "上传文件路径")
	flag.Parse()

	fileInfo, err := os.Stat(*file_path)
    if err != nil {
        fmt.Println(err)
        return
    }
    fileSize := fileInfo.Size()
	fileName := filepath.Base(*file_path)
	file_extension := filepath.Ext(fileName)[1]
	fileType := getFileType(fileName)

	fileHash, err := getFileSHA1(*file_path)
	if err != nil {
	fmt.Println("Error:", err)
		return
	}
	init_url := "http://uploadapi2.stariverpan.com:18090/v2/file/init"
	method := "POST"

	payload := strings.NewReader(fmt.Sprintf(`{"hash":"%s","fileHash":"%s","fileName":"%s","fileSize":%d,"fileCid":"","fileState":0,"parentId":"0","chunkSize":0,"suffix":"","partList":[],"accessToken":"%s"}`,fileHash,fileHash,fileName,fileSize,*token))
	client := &http.Client {
	}
	req, err := http.NewRequest(method, init_url, payload)

	if err != nil {
	fmt.Println(err)
		return
	}
	req.Header.Add("Accept", "application/json, text/plain, */*")
	req.Header.Add("Accept-Language", "zh")
	req.Header.Add("Authorization", fmt.Sprintf("Bearer %s",*token))
	req.Header.Add("Content-Type", "application/json;charset=UTF-8")
	req.Header.Add("Custom-Agent", "PC")
	req.Header.Add("Proxy-Connection", "keep-alive")
	req.Header.Add("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) xiaolongyunpan/3.2.7 Chrome/100.0.4896.143 Electron/18.2.0 Safari/537.36")

	res, err := client.Do(req)
	if err != nil {
	fmt.Println(err)
		return
	}
	defer res.Body.Close()

	body, err := ioutil.ReadAll(res.Body)
	if err != nil {
	fmt.Println(err)
		return
	}
	var init_res InitResponse
    jsonerr := json.Unmarshal([]byte(string(body)), &init_res)
    if jsonerr != nil {
        fmt.Println("JSON解析错误：", jsonerr)
    }
	if (len(init_res.Data.FileCid) != 0){
		add_url := "https://productapi.stariverpan.com/cmsprovider/v2.5/cloud/add-file"
		add_method := "POST"
		add_payload := strings.NewReader(fmt.Sprintf(`{"filePath":"","dirPath":[],"fileName":"%s","fileSize":%d,"fileCid":"%s","fileType":4,"parentId":"0","suffix":"file_extension","thumbnail":"","duration":1,"width":"0","height":"0"}`,fileName,fileSize,init_res.Data.FileCid,file_extension))
		addd_req, err := http.NewRequest(add_method, add_url, add_payload)

		if err != nil {
			fmt.Println(err)
			return
		}
		addd_req.Header.Add("Host", "productapi.stariverpan.com")
		addd_req.Header.Add("accept", "application/json, text/plain, */*")
		addd_req.Header.Add("Authorization", fmt.Sprintf("Bearer %s",*token))
		addd_req.Header.Add("custom-agent", "PC")
		addd_req.Header.Add("accept-language", "zh")
		addd_req.Header.Add("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) xiaolongyunpan/3.2.7 Chrome/100.0.4896.143 Electron/18.2.0 Safari/537.36")
		addd_req.Header.Add("content-type", "application/json;charset=UTF-8")
		addd_req.Header.Add("sec-fetch-site", "cross-site")
		addd_req.Header.Add("sec-fetch-mode", "cors")
		addd_req.Header.Add("sec-fetch-dest", "empty")
	

		add_res, add_err := client.Do(addd_req)
		if add_err != nil {
			fmt.Println(add_err)
			return
		}
		defer add_res.Body.Close()

		add_body, err := ioutil.ReadAll(add_res.Body)
		if err != nil {
			fmt.Println(err)
			return
		}
		fmt.Println(string(add_body))
		return
	}

	fmt.Println(init_res)
	chunkSize := init_res.Data.ChunkSize

	chunks := int(fileSize) / chunkSize
    if fileSize%int64(chunkSize) != 0 {
        chunks++
    }
    // 打开文件
    file, err := os.Open(*file_path)
    if err != nil {
        fmt.Println(err)
        return
    }
    defer file.Close()

    // 分片上传文件
    for i := 0; i < chunks; i++ {
		
        // 读取分片数据
        offset := int64(i * chunkSize)
        limit := int64(chunkSize)
        if offset+limit > fileSize {
            limit = fileSize - offset
        }
        chunkData := make([]byte, limit)

		if (len(chunkData)<chunkSize){
			chunkSize = len(chunkData)
		}

        _, err := file.ReadAt(chunkData, offset)
        if err != nil {
            fmt.Println(err)
            return
        }
		chunkHash := getChunkSHA1(chunkData)
        // 构造http请求
        requestBody := new(bytes.Buffer)
        writer := multipart.NewWriter(requestBody)
        // 添加文件分片
        part, err := writer.CreateFormFile("file", filepath.Base(*file_path))
        if err != nil {
            fmt.Println(err)
            return
        }
        part.Write(chunkData)

        // 添加分片信息
		serverUrl := fmt.Sprintf("%s/v2/file/chunk/upload-binary?fileHash=%s&accessToken=%s&chunkIndex=%d&chunkHash=%s&chunkSize=%d",init_res.Data.UploadEp,fileHash,*token,i,chunkHash,chunkSize)
        
		req, err := http.NewRequest("POST", serverUrl, bytes.NewBuffer(chunkData))
		if err != nil {
			fmt.Println("Error creating request:", err)
			return
		}
		req.Header.Set("Content-Type", "application/octet-stream")
		client := &http.Client{}
		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("Error sending request:", err)
			return
		}
		defer resp.Body.Close()
		body, err := ioutil.ReadAll(resp.Body)
        fmt.Println(string(body))
    }

	slice_url := fmt.Sprintf("%s/v2/file/chunk/splice",init_res.Data.UploadEp)
	method = "POST"

	payload = strings.NewReader(fmt.Sprintf(`{"appEnv":"prod","fileName":"%s","fileHash":"%s","accessToken":"%s","passThrough":"{\"dirPath\":[],\"duration\":1,\"parentId\":\"0\",\"fileType\":%d,\"width\":\"0\",\"height\":\"0\"}","noCallback":false}`,fileName,fileHash,*token,fileType))
	fmt.Println(payload)


	slic_req, err := http.NewRequest(method, slice_url, payload)
	if err != nil {
		fmt.Println(err)
		return
	}

	parsedUrl, err := url.Parse(init_res.Data.UploadEp)
    if err != nil {
        panic(err)
    }

    host := parsedUrl.Hostname()
    port := parsedUrl.Port()
	slic_req.Header.Add("Host", fmt.Sprintf("%s:%d",host,port))
	slic_req.Header.Add("Accept", "application/json, text/plain, */*")
	slic_req.Header.Add("Authorization", fmt.Sprintf("Bearer %s",*token))
	slic_req.Header.Add("Custom-Agent", "PC")
	slic_req.Header.Add("Accept-Language", "zh")
	slic_req.Header.Add("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) xiaolongyunpan/3.2.7 Chrome/100.0.4896.143 Electron/18.2.0 Safari/537.36")
	slic_req.Header.Add("Content-Type", "application/json;charset=UTF-8")

	slic_res, err := client.Do(slic_req)
	if err != nil {
		fmt.Println(err)
		return
	}
	defer slic_res.Body.Close()

	slice_body, err := ioutil.ReadAll(slic_res.Body)
	if err != nil {
		fmt.Println(err)
		return
	}
	fmt.Println(string(slice_body))

    
}


func getFileSHA1(filename string) (string, error) {
	// 打开文件
	f, err := os.Open(filename)
	if err != nil {
	  return "", err
	}
	defer f.Close()
  
	// 创建SHA1哈希对象
	h := sha1.New()
  
	// 将文件内容写入哈希对象
	if _, err := io.Copy(h, f); err != nil {
	  return "", err
	}
  
	// 获取哈希值并返回
	sha1sum := fmt.Sprintf("%x", h.Sum(nil))
	return sha1sum, nil
  }
  
  func getChunkSHA1(body []byte) string {
    hasher := sha1.New()
    hasher.Write(body)
    result := hasher.Sum(nil)
    resultString := fmt.Sprintf("%x", result)
    return resultString
}

func getFileType(fileName string) int {
    var fileTypeMap = map[string]int{
        "txt": 1,
        "jpeg": 2,
        "jpg": 2,
        "gif": 2,
        "bmp": 2,
        "png": 2,
        "avif": 2,
        "heic": 2,
        "mp4": 3,
        "mkv": 3,
        "m4u": 3,
        "m4v": 3,
        "mov": 3,
        ".3gp": 3,
        "asf": 3,
        "avi": 3,
        "wmv": 3,
        "flv": 3,
        "mpe": 3,
        "mpeg": 3,
        "mpg": 3,
        "mpg4": 3,
        "mpeg4": 3,
        "mpga": 3,
        "rmvb": 3,
        "rm": 3,
        "aac": 4,
        "ogg": 4,
        "wav": 4,
        "wma": 4,
        "m3u": 4,
        "m4a": 4,
        "m4b": 4,
        "m4p": 4,
        "m4r": 4,
        "mp2": 4,
        "mp3": 4,
        "bin": 5,
        "class": 5,
        "conf": 5,
        "cpp": 5,
        "c": 5,
        "exe": 5,
        "gtar": 5,
        "gz": 5,
        "h": 5,
        "htm": 5,
        "html": 5,
        "jar": 5,
        "java": 5,
        "js": 5,
        "log": 5,
        "mpc": 5,
        "msg": 5,
        "pps": 5,
        "prop": 5,
        "rc": 5,
        "rtf": 5,
        "sh": 5,
        "tar": 5,
        "tgz": 5,
        "wps": 5,
        "xml": 5,
        "z": 5,
        "zip": 5,
        "apk": 5,
        "ipa": 5,
        "app": 5,
        "hap": 5,
        "docx": 6,
        "doc": 6,
        "xls": 7,
        "xlsx": 7,
        "ppt": 8,
        "pptx": 8,
        "pdf": 9,
        "epub": 11,
    }
    fileExt := getFileExtensionName(fileName)
    if fileType, ok := fileTypeMap[fileExt]; ok {
        return fileType
    } else {
        return 5
    }
}

func getFileExtensionName(fileName string) string {
    index := strings.LastIndex(fileName, ".")
    if index == -1 || index == len(fileName)-1 {
        return ""
    }
    return fileName[index+1:]
}