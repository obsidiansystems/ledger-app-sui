# \DefaultApi

All URIs are relative to *http://127.0.0.1:5000*

Method | HTTP request | Description
------------- | ------------- | -------------
[**apdu_post**](DefaultApi.md#apdu_post) | **Post** /apdu | Transmit APDU and return device response
[**automation_post**](DefaultApi.md#automation_post) | **Post** /automation | Updates the automation rules
[**button_button_post**](DefaultApi.md#button_button_post) | **Post** /button/{button} | Press or release a button (Nano S and Nano X)
[**events_delete**](DefaultApi.md#events_delete) | **Delete** /events | Reset the list of events
[**events_get**](DefaultApi.md#events_get) | **Get** /events | Get the events produced by the app
[**finger_post**](DefaultApi.md#finger_post) | **Post** /finger | Touch the screen (Blue)
[**screenshot_get**](DefaultApi.md#screenshot_get) | **Get** /screenshot | Get a screenshot



## apdu_post

> crate::models::Apdu apdu_post(apdu)
Transmit APDU and return device response

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**apdu** | [**Apdu**](Apdu.md) |  | [required] |

### Return type

[**crate::models::Apdu**](Apdu.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## automation_post

> automation_post(body)
Updates the automation rules

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body** | **serde_json::Value** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## button_button_post

> button_button_post(button, button)
Press or release a button (Nano S and Nano X)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**button** | [**crate::models::ButtonName**](.md) | Button to press | [required] |
**button** | [**Button**](Button.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## events_delete

> events_delete()
Reset the list of events

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## events_get

> String events_get(stream)
Get the events produced by the app

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**stream** | Option<**bool**> | Stream the events instead of returning a list |  |[default to false]

### Return type

**String**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## finger_post

> finger_post(finger)
Touch the screen (Blue)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**finger** | [**Finger**](Finger.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## screenshot_get

> std::path::PathBuf screenshot_get()
Get a screenshot

### Parameters

This endpoint does not need any parameter.

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: image/png

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

