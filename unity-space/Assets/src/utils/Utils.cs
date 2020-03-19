using UnityEngine;
#if UNITY_EDITOR
using UnityEditor;
#endif
	
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.Serialization.Formatters.Binary;
using System.IO;

namespace utils
{

    public static class Utils
    {
        public static string Uuid()
        {
            return System.Guid.NewGuid().ToString();
        }

        public static T Inst<T>(GameObject prefab) where T : Component
        {
            var obj = Inst(prefab);
            return obj.GetComponent<T>();
        }

        public static GameObject Inst(GameObject prefab, Vector3 position, Quaternion rotation)
        {
            var obj = Inst(prefab);
            obj.transform.position = position;
            obj.transform.rotation = rotation;
            return obj;
        }

        public static GameObject Inst(GameObject prefab)
        {
            if (prefab == null)
            {
                Debug.LogWarning("Could not create a new instance of a null prefab");
                return null;
            }

#if UNITY_EDITOR
            GameObject obj;
            if (IsPrefab(prefab))
                obj = PrefabUtility.InstantiatePrefab(prefab) as GameObject;
            else
                obj = (GameObject.Instantiate(prefab) as GameObject);
#else
		var obj = (GameObject.Instantiate(prefab) as GameObject);
#endif
            obj.name = prefab.name;
            return obj;
        }

        public static bool IsPrefab(GameObject obj)
        {
#if UNITY_EDITOR
            // var isPrefab = PrefabUtility.GetPrefabParent(obj) == null && PrefabUtility.GetPrefabObject(obj) != obj;
            var isPrefab = PrefabUtility.GetPrefabType(obj) != PrefabType.None;
            return isPrefab;
#else
		Debug.LogWarning("Invalid request for runtime");
		return false;
#endif
        }

        public static T Inst<T>(T prefab) where T : Component
        {
#if UNITY_EDITOR
            T comp;
            if (IsPrefab(prefab.gameObject))
                comp = PrefabUtility.InstantiatePrefab(prefab) as T;
            else
                comp = (GameObject.Instantiate(prefab) as T);
#else
		var comp = (GameObject.Instantiate(prefab) as T);
#endif
            comp.name = prefab.name;
            return comp;
        }

        public static T Inst<T>(GameObject prefab, Vector3 pos, Quaternion rot) where T : Component
        {
            var obj = Inst(prefab);
            obj.transform.position = pos;
            obj.transform.rotation = rot;
            obj.name = prefab.name;
            return obj.GetComponent<T>();
        }

        public static T Inst<T>(T prefab, Vector3 pos, Quaternion rot) where T : Component
        {
            var obj = Inst(prefab);
            obj.transform.position = pos;
            obj.transform.rotation = rot;
            obj.name = prefab.name;
            return obj;
        }

        public static void Destroy(GameObject gameObject, bool now = false)
        {
            var immediate = !Application.isPlaying || now;
            if (immediate) GameObject.DestroyImmediate(gameObject);
            else GameObject.Destroy(gameObject);
        }

        public static void DestroyComponent(Component c)
        {
            if (Application.isPlaying)
                Component.Destroy(c);
            else
                Component.DestroyImmediate(c);
        }

        public static void DestroyChild(this Transform self, string name)
        {
            var t = self.Find(name);
            if (t) Destroy(t.gameObject);
        }


        public static Vector3 SetZ(Vector3 v, float z)
        {
            v.z = z;
            return v;
        }

        public static Vector3 SetX(Vector3 v, float x)
        {
            v.x = x;
            return v;
        }

        public static Vector3 SetY(Vector3 v, float y)
        {
            v.y = y;
            return v;
        }

        public static Vector3 WithZ(this Vector3 v, float z)
        {
            v.z = z;
            return v;
        }

        public static Vector3 WithX(this Vector3 v, float x)
        {
            v.x = x;
            return v;
        }

        public static Vector3 WithY(this Vector3 v, float y)
        {
            v.y = y;
            return v;
        }

        public static Vector2 WithX(this Vector2 v, float x)
        {
            v.x = x;
            return v;
        }

        public static Vector2 WithY(this Vector2 v, float y)
        {
            v.y = y;
            return v;
        }

        public static bool IsSameDir(Vector3 v1, Vector3 v2)
        {
            return Vector3.Dot(v1, v2) > 0f;
        }

        public static void CleanUp(Transform transform)
        {
            if (!transform)
            {
                Debug.LogWarning("Could not CleanUp a invalid transform");
                return;
            }

            var l = new List<Transform>();
            foreach (Transform t in transform) l.Add(t);

            if (Application.isPlaying)
            {
                foreach (var t in l)
                {
                    t.parent = null;
                    GameObject.Destroy(t.gameObject);
                }
            }
            else
            {
                foreach (var t in l)
                {
                    t.parent = null;
                    GameObject.DestroyImmediate(t.gameObject);
                }
            }
        }

        public static void CleanUpNow(Transform transform)
        {
            var l = new List<Transform>();
            foreach (Transform t in transform) l.Add(t);
            foreach (var t in l)
            {
                t.parent = null;
                GameObject.DestroyImmediate(t.gameObject);
            }
        }

        public static void CleanUp<T>() where T : Component
        {
            var list = GameObject.FindObjectsOfType(typeof(T));
            foreach (var l in list) GameObject.Destroy(l);
        }

        public static string ToString(System.Object obj)
        {
            if (obj == null) return "null";
            if (obj is Dictionary<string, string>)
            {
                Dictionary<string, string> dick = obj as Dictionary<string, string>;
                if (dick.Count == 0) return "{}";
                var b = "{";
                foreach (var e in dick)
                {
                    b += e.Key + "=" + ToString(e.Value) + ", ";
                }
                return b.Substring(0, b.Length - 2) + "}";
                //		} else if (obj.GetType(). {
                //			List<_> list = obj as List<_>;
                //			if (list.Count == 0) return "[]";
                //			var b = "[";
                //			foreach(object e in list) {
                //				b += ToString(e) + ", ";
                //			}
                //			return b.Substring(0, b.Length - 2)+ "]";
            }
            else if (obj is HashSet<string>)
            {
                var set = obj as HashSet<string>;
                return ToString(set.ToArray());
            }
            else if (obj.GetType().IsArray)
            {
                // TODO this don't fuck work
                var array = (object[])obj;
                if (array.Length == 0) return "[]";
                var b = "[";
                foreach (var i in array)
                {
                    b += ToString(i) + ", ";
                }
                return b.Substring(0, b.Length - 2) + "]";
            }
            else if (obj is List<string>)
            {
                // TODO this don't fuck work
                var list = (List<string>)obj;
                if (list.Count == 0) return "[]";
                var b = "[";
                foreach (var i in list)
                {
                    b += i + ", ";
                }
                return b.Substring(0, b.Length - 2) + "]";
            }
            else
            {
                var sb = new System.Text.StringBuilder();
                var type = obj.GetType();
                sb.Append(type.Name);
                sb.Append("(");
                var fields = type.GetFields();
                if (fields.Length > 0)
                {
                    foreach (System.Reflection.FieldInfo field in fields)
                    {
                        sb.Append(field.Name);
                        sb.Append(": ");
                        sb.Append(field.GetValue(obj));
                        sb.Append(",");
                    }
                    sb.Remove(0, sb.Length - 1);
                }
                sb.Append(")");
                return sb.ToString();
            }
        }

        public static string Replace(string msg, params object[] keysAndValues)
        {
            for (var i = 0; i < keysAndValues.Length - 1; i = i + 2)
            {
                string key = "{" + System.Convert.ToString(keysAndValues[i]) + "}";
                string value = System.Convert.ToString(keysAndValues[i + 1]);
                msg = msg.Replace(key, value);
            }
            return msg;
        }

        public static void SetParentZero(Transform child, Transform parent)
        {
            child.parent = parent;
            child.localPosition = Vector3.zero;
            child.localScale = Vector3.one;
            child.localRotation = Quaternion.identity;
        }

        public static void SetParentKeepTransform(Transform child, Transform parent)
        {
            var p = child.position;
            var r = child.rotation;
            var s = child.localScale;

            child.parent = parent;

            child.localPosition = p;
            child.localRotation = r;
            child.localScale = s;
        }

        public static T FindComponent<T>(this GameObject obj)
        {
            System.Type type = typeof(T);
            Component[] components = obj.GetComponents<Component>();
            foreach (Component c in components)
            {
                if (type.IsAssignableFrom(c.GetType()))
                {
                    // cheat the compiller to transform a component into T
                    object o = (object)c;
                    return (T)o;
                }
            }
            return default(T);
        }

        public static List<T> FindComponents<T>(this GameObject obj)
        {
            System.Type type = typeof(T);
            List<T> list = new List<T>();
            Component[] components = obj.GetComponents<Component>();
            foreach (Component c in components)
            {
                if (type.IsAssignableFrom(c.GetType()))
                {
                    // cheat the compiller to transform a component into T
                    object o = (object)c;
                    list.Add((T)o);
                }
            }
            return list;
        }

        public static void SetGroup(string groupName, Transform t)
        {
            t.parent = GetGroup(groupName);
        }

        public static Transform GetGroup(string groupName)
        {
            var group = GameObject.Find(groupName);
            if (group == null)
            {
                var index = groupName.IndexOf('/');
                if (index < 0)
                {
                    group = new GameObject(groupName);
                }
                else
                {
                    var array = groupName.Split('/');
                    Transform last = null;
                    string predicate = "/";
                    for (int i = 0; i < array.Length; i++)
                    {
                        var obj = GameObject.Find(predicate + array[i]);
                        if (obj == null)
                        {
                            obj = new GameObject(array[i]);
                            //                        Debug.Log("Creating "+array[i]);
                        }
                        else
                        {
                            //                        Debug.Log("Found "+array[i]);
                        }
                        if (last)
                        {
                            obj.transform.parent = last;
                        }

                        last = obj.transform;
                        group = obj;
                        predicate += array[i] + "/";
                    }
                    //                Debug.Log("Last "+group);
                }
            }
            return group.transform;
        }

        public static T CopyComponent<T>(T original, GameObject destination) where T : Component
        {
            System.Type type = original.GetType();
            Component copy = destination.AddComponent(type);
            System.Reflection.FieldInfo[] fields = type.GetFields();
            foreach (System.Reflection.FieldInfo field in fields)
            {
                field.SetValue(copy, field.GetValue(original));
            }
            return copy as T;
        }

        public static void CopyField(object from, object to)
        {
            System.Reflection.FieldInfo[] fields = from.GetType().GetFields();
            foreach (System.Reflection.FieldInfo field in fields)
            {
                field.SetValue(to, field.GetValue(from));
            }
        }

        public static Vector3 RandomCircleXZ()
        {
            var angle = UnityEngine.Random.Range(0.0f, 360.0f);
            var rotation = Quaternion.Euler(0f, angle, 0f);
            return rotation * Vector3.forward;
        }

        // @site http://forum.unity3d.com/threads/52930-Vector3.Angle%28%29
        public static float Angle(Vector3 fwd, Vector3 targetDir, Vector3 upDir)
        {
            var angle = Vector3.Angle(fwd, targetDir);
            angle *= AngleDir(fwd, targetDir, upDir);
            return angle;
        }

        //returns -1 when to the left, 1 to the right, and 0 for forward/backward
        public static int AngleDir(Vector3 fwd, Vector3 targetDir, Vector3 up)
        {
            Vector3 perp = Vector3.Cross(fwd, targetDir);
            float dir = Vector3.Dot(perp, up);

            if (dir > 0f)
            {
                return 1;
            }
            else if (dir < 0f)
            {
                return -1;
            }
            else
            {
                return 0;
            }
        }

        public static float Angle360(float angle)
        {
            while (angle > 360f) angle -= 360f;
            while (angle < 360f) angle += 360f;
            return angle;
        }

        public static Vector3 AvaragePosition<T>(List<T> list) where T : Component
        {
            if (list.Count == 0) return Vector3.zero;

            int total = list.Count;
            int count = 0;
            Vector3 sum = Vector3.zero;
            for (var i = 0; i < total; i++)
            {
                if (!list[i]) continue;
                count++;
                sum += list[i].transform.position;
            }

            var avg = sum / count;
            return avg;
        }

        public static List<T> ToList<T>(HashSet<T> set)
        {
            var l = new List<T>(set.Count);
            foreach (var t in set) l.Add(t);
            return l;
        }


        public static HashSet<T> ToSet<T>(List<T> list)
        {
            var set = new HashSet<T>();
            foreach (var t in list) set.Add(t);
            return set;
        }

        public static List<T> ToList<T>(params T[] t)
        {
            return new List<T>(t);
        }

        public static List<T> Enumerable2List<T>(System.Collections.IEnumerable e)
        {
            var list = new List<T>();
            var i = e.GetEnumerator();
            while (i.MoveNext())
            {
                list.Add((T)i.Current);
            }
            return list;
        }

        public static List<K> Collect<T, K>(this List<T> list, System.Func<T, K> fuck)
        {
            List<K> result = new List<K>();
            for (int i = 0, max = list.Count; i < max; i++)
            {
                K k = fuck(list[i]);
                if (k != null) result.Add(k);
            }
            return result;
        }

        public static List<K> FlatMap<T, K>(this List<T> list, System.Func<T, List<K>> fuck)
        {
            List<K> result = new List<K>();
            for (int i = 0, max = list.Count; i < max; i++)
            {
                List<K> list2 = fuck(list[i]);
                for (int j = 0, max2 = list2.Count; j < max2; j++)
                {
                    result.Add(list2[j]);
                }
            }
            return result;
        }

        public static Transform SearchChild(this Transform t, string name)
        {
            if (!t) return null;

            for (var i = 0; i < t.childCount; i++)
            {
                var t1 = t.GetChild(i);
                if (t1.name == name) return t1;

                t1 = SearchChild(t1, name);
                if (t1) return t1;
            }

            return null;
        }

        public static Bounds BoundsFor(Vector3 min, Vector3 max)
        {
            var b = new Bounds(min, Vector3.zero);
            b.Encapsulate(max);
            return b;
        }

        public static Bounds BoundsColliders(GameObject obj)
        {
            var colliders = obj.GetComponentsInChildren<Collider2D>();
            if (colliders.Length == 0)
                return new Bounds(obj.transform.position, Vector3.zero);

            var bounds = colliders[0].bounds;
            for (int i = 1, max = colliders.Length; i < max; i++)
            {
                bounds.Encapsulate(colliders[i].bounds);
            }
            return bounds;
        }

        public static Bounds BoundsRenderes(GameObject obj)
        {
            var renderers = obj.GetComponentsInChildren<Renderer>();
            if (renderers.Length == 0)
                return new Bounds(obj.transform.position, Vector3.zero);

            var bounds = renderers[0].bounds;
            for (int i = 1, max = renderers.Length; i < max; i++)
            {
                bounds.Encapsulate(renderers[i].bounds);
            }
            return bounds;
        }

        public static int IndexOf(this Transform parent, Transform child)
        {
            int index = -1;
            for (int i = 0, total = parent.childCount; i < total; i++)
            {
                if (parent.GetChild(i) == child)
                {
                    index = i;
                    break;
                }
            }
            return index;
        }

        public static bool IsChildActive(Transform parent)
        {
            var total = parent.childCount;
            for (var i = 0; i < total; i++)
            {
                if (parent.GetChild(i).gameObject.activeInHierarchy)
                {
                    return true;
                }
            }
            return false;
        }

        public static LayerMask ToLayerMask(int layer)
        {
            return 1 << layer;
        }

        public static bool InLayerMask(GameObject obj, LayerMask layerMask)
        {
            int objLayerMask = (1 << obj.layer);
            return (layerMask.value & objLayerMask) > 0;
        }

        public static Vector3 Rand(Bounds b)
        {
            return new Vector3(Random.Range(b.min.x, b.max.x), Random.Range(b.min.y, b.max.y), Random.Range(b.min.z, b.max.z));
        }

        public static Quaternion LookAt2d(Vector2 dir)
        {
            float angle = Mathf.Atan2(dir.y, dir.x) * Mathf.Rad2Deg;
            return Quaternion.AngleAxis(angle, Vector3.forward);
        }

        // WARNING: This shit don't work
        public static IEnumerator WaitForAnimation(Animator animator, System.Action callback)
        {
            // wait animation to start
            float length;
            do
            {
                yield return 0f;
                length = animator.GetCurrentAnimatorStateInfo(0).length;
            } while (length == 0f);

            // wait for animation to finish
            if (callback != null)
            {
                while (animator.GetCurrentAnimatorStateInfo(0).normalizedTime < 1f)
                {
                    callback();
                    yield return 0f;
                }
            }
            else
            {
                yield return new WaitForSeconds(length);
            }
        }

        public static List<Transform> GetActiveChildrensOrderByName(Transform transform)
        {
            List<Transform> list = new List<Transform>();
            foreach (Transform t in transform)
            {
                if (t.gameObject)
                {
                    list.Add(t);
                }
            }
            list.Sort((a, b) => a.name.CompareTo(b.name));
            return list;
        }

        public static Vector2 V2(Vector3 v)
        {
            return new Vector2(v.x, v.y);
        }

        public static string ToBase64(byte[] bytes)
        {
            return System.Convert.ToBase64String(bytes);
        }

        public static byte[] FromBase64(string str)
        {
            return System.Convert.FromBase64String(str);
        }

        public static byte[] SerializeBinary(object obj)
        {
            var b = new BinaryFormatter();
            // Create an in memory stream
            var m = new MemoryStream();
            // Save the scores
            b.Serialize(m, obj);
            // To String
            return m.GetBuffer();
        }

        public static T ParseBinary<T>(byte[] str)
        {
            //Binary formatter for loading back
            var b = new BinaryFormatter();
            //Create a memory stream with the data
            var m = new MemoryStream(str);
            //Load back the scores
            return (T)b.Deserialize(m);
        }

        //	public static List<string> GetPathItem(Transform transform, List<string> path)
        //	{
        //		path.Add(transform.name);
        //		if (transform.parent)
        //			return GetPathItem(transform.parent, path);
        //		return path;
        //	}
        //
        //	public static List<string> GetPath(Transform transform)
        //	{
        //		return new List<string>();
        //	}

        public static IEnumerator RealtimeWait(float time)
        {
            float waitTime = Time.realtimeSinceStartup + time;
            while (Time.realtimeSinceStartup < waitTime)
            {
                yield return 0;
            }
        }

        //	public static List<T> ToList<T>(this IEnumerable array) 
        //	{
        //		return new List<T>(array);
        //	}

        public static T Find<T>() where T : Object
        {
            return (T)GameObject.FindObjectOfType<T>();
        }

        public static List<T> FindAll<T>() where T : Object
        {
            return new List<T>(GameObject.FindObjectsOfType<T>());
        }

        public static void Assert(bool b)
        {
            if (!b) throw new UnityException("AssertException");
        }

        public static void Assert(bool b, string msg, params object[] args)
        {
            if (!b) throw new UnityException(string.Format(msg, args));
        }

        public static T RandomOne<T>(List<T> list, bool remove = false)
        {
            if (list.Count == 0) return default(T);
            var i = UnityEngine.Random.Range(0, list.Count);
            var o = list[i];
            if (remove) list.RemoveAt(i);
            return o;
        }

        public static T RandomOther<T>(List<T> list, T obj)
        {
            if (list.Count == 0) return default(T);

            if (list.Count == 1) return list[0];

            var clone = new List<T>(list);

            while (clone.Count > 0)
            {
                var i = Random.Range(0, clone.Count);
                if (clone[i].Equals(obj))
                    clone.RemoveAt(i);
                else
                    return clone[i];
            }

            throw new UnityException();
        }

        public static T[] GetValues<T>()
        {
            return (T[])System.Enum.GetValues(typeof(T));
        }

        public static T ParseEnum<T>(string str)
        {
            return (T)System.Enum.Parse(typeof(T), str);
        }

        public static string EnumToString(System.Enum value)
        {
            return value.ToString();
        }

        public static void NotImplemented(string msg = "")
        {
            throw new System.NotImplementedException(msg);
        }

        public static void Error(string msg, params object[] args)
        {
            if (msg == null)
                throw new UnityException();
            else
                throw new UnityException(string.Format(msg, args));
        }

        public static void Error()
        {
            throw new UnityException();
        }

        public static void IllegalArgument(string msg = "")
        {
            throw new System.ArgumentException(msg);
        }

        // http://stackoverflow.com/questions/2912340/c-sharp-hashcode-builder	
        public static int ComputeHashFrom(params object[] obj)
        {
            ulong res = 0;
            for (uint i = 0; i < obj.Length; i++)
            {
                object val = obj[i];
                res += val == null ? i : (ulong)val.GetHashCode() * (1 + 2 * i);
            }
            return (int)(uint)(res ^ (res >> 32));
        }

        public static T FindNearest<T>(Vector3 pos, List<T> list, System.Func<T, Vector3> mapFunc) where T : Component
        {
            if (list == null || list.Count == 0)
                return null;

            if (list.Count == 1)
                return list[0];

            var minIndex = -1;
            float minValue = float.MaxValue;
            var total = list.Count;
            for (var i = 0; i < total; i++)
            {
                if (!list[i]) continue;

                float v = (mapFunc(list[i]) - pos).sqrMagnitude;
                if (v < minValue)
                {
                    minIndex = i;
                    minValue = v;
                }
            }

            if (minIndex == -1) return null;

            return list[minIndex];
        }

        public static int Count<T>(List<T> list, System.Func<T, int> func)
        {
            int count = 0;
            for (int i = 0, total = list.Count; i < total; i++)
            {
                count += func(list[i]);
            }
            return count;
        }

        public static int Count<T>(List<T> list, System.Predicate<T> func)
        {
            int count = 0;
            for (int i = 0, total = list.Count; i < total; i++)
            {
                if (func(list[i])) count++;
            }
            return count;
        }

        public static float Sum<T>(List<T> list, System.Func<T, float> func)
        {
            float sum = 0;
            for (int i = 0, total = list.Count; i < total; i++)
            {
                sum += func(list[i]);
            }
            return sum;
        }

        public static int SumI<T>(List<T> list, System.Func<T, int> func)
        {
            int sum = 0;
            for (int i = 0, total = list.Count; i < total; i++)
            {
                sum += func(list[i]);
            }
            return sum;
        }

        // @see http://answers.unity3d.com/questions/189724/polar-spherical-%20%20%20%20%20%20%20%20%20%20%20%20%20%20coordinates-to-xyz-and-vice-versa.html
        public static Vector2 CartesianToPolar(Vector3 point)
        {
            Vector2 polar;
            //calc longitude
            polar.y = Mathf.Atan2(point.x, point.z);
            //this is easier to write and read than sqrt(pow(x,2), pow(y,2))!
            var xzLen = new Vector2(point.x, point.z).magnitude;
            //atan2 does the magic
            polar.x = Mathf.Atan2(-point.y, xzLen);
            //convert to deg
            polar *= Mathf.Rad2Deg;
            return polar;
        }

        public static T MinBy<T>(List<T> list, System.Func<T, float> Func)
        {
            var index = 0;
            var minValue = Func(list[0]);
            for (int i = 1, max = list.Count; i < max; i++)
            {
                var o = list[i];
                var v = Func(o);

                if (v < minValue)
                {
                    index = i;
                    minValue = v;
                }
            }

            return list[index];
        }

        public static T MaxBy<T>(List<T> list, System.Func<T, float> Func)
        {
            var index = 0;
            var maxValue = Func(list[0]);
            for (int i = 1, max = list.Count; i < max; i++)
            {
                var o = list[i];
                var v = Func(o);

                if (v > maxValue)
                {
                    index = i;
                    maxValue = v;
                }
            }

            return list[index];
        }

        // @see http://answers.unity3d.com/questions/189724/polar-spherical-%20%20%20%20%20%20%20%20%20%20%20%20%20%20coordinates-to-xyz-and-vice-versa.html
        public static Vector3 PolarToCartesian(Vector2 polar)
        {
            //an origin vector, representing lat,lon of 0,0.
            var origin = new Vector3(0, 0, 1);
            //build a quaternion using euler angles for lat,lon
            var rotation = Quaternion.Euler(polar.x, polar.y, 0);
            //transform our reference vector by the rotation. Easy-peasy!
            var point = rotation * origin;
            return point;
        }

        public class DiffResult<T>
        {
            public bool onlyA;
            public bool onlyB;
            public T obj;

            public DiffResult(bool onlyA, bool onlyB, T obj)
            {
                this.onlyA = onlyA;
                this.onlyB = onlyB;
                this.obj = obj;
            }
        }

        /*
            This bad impl is sufficient for now, but ideally it should return 
            the itens in both lists without repeat.
        */
        public static List<DiffResult<T>> Diff<T>(List<T> listA, List<T> listB)
        {
            HashSet<T> setA = ToSet(listA);
            var result = new List<DiffResult<T>>(listB.Count);
            foreach (var i in listB)
            {
                if (!setA.Contains(i))
                {
                    result.Add(new DiffResult<T>(false, true, i));
                }
            }

            HashSet<T> setB = ToSet(listB);
            foreach (var i in listA)
            {
                if (!setB.Contains(i))
                {
                    result.Add(new DiffResult<T>(true, false, i));
                }
            }

            return result;
        }

        public static Vector3 RaycastPoint(Plane plane, Ray ray)
        {
            float distance;
            plane.Raycast(ray, out distance);
            return ray.GetPoint(distance);
        }

        public static float RandomPercent(float value, float percent)
        {
            var valueInPercent = value * percent;
            return Random.Range(value - valueInPercent, value + percent);
        }

        public static V GetOrElse<K, V>(this Dictionary<K, V> dic, K key, V defaultValue)
        {
            V output;
            if (dic.TryGetValue(key, out output))
                return output;
            return defaultValue;
        }

        public static void SetDirt(Object mono)
        {
#if UNITY_EDITOR
            if (!Application.isPlaying)
            {
                EditorUtility.SetDirty(mono);
            }
#endif
        }

        public static bool IsInTransitionTo(this Animator animator, string stateName)
        {
            if (!animator.IsInTransition(0)) return false;
            var fullName = "Base Layer." + stateName;
            var hash = Animator.StringToHash(fullName);
            var state = animator.GetNextAnimatorStateInfo(0);
            return state.nameHash == hash;
        }

        public static bool IsState(this Animator animator, string stateName)
        {
            var fullName = "Base Layer." + stateName;
            var hash = Animator.StringToHash(fullName);
            var state = animator.GetCurrentAnimatorStateInfo(0);
            return state.nameHash == hash;
        }

        public static void ReplaceChild(Transform parent, string name, GameObject newChild)
        {
            var child = parent.Find(name);
            if (child) Destroy(child.gameObject);

            SetParentZero(newChild.transform, parent);
            newChild.name = name;
        }

        // http://stackoverflow.com/questions/2601477/dictionary-returning-a-default-value-if-the-key-does-not-exist
        public static TValue GetValueOrDefault<TKey, TValue>(this IDictionary<TKey, TValue> dictionary, TKey key, System.Func<TValue> defaultValueProvider)
        {
            TValue value;
            return dictionary.TryGetValue(key, out value) ? value : defaultValueProvider();
        }

        public static List<T> Clone<T>(this List<T> list)
        {
            if (list == null)
                return null;
            return new List<T>(list);
        }

        public static void ForEach<T>(T t, System.Action<T> Callback)
        {
            if (t != null) Callback(t);
        }

        public static List<T> FindChildren<T>(this MonoBehaviour self) where T : Object
        {
            return new List<T>(self.GetComponentsInChildren<T>());
        }

        public static T FindInChildren<T>(this MonoBehaviour self) where T : Object
        {
            return self.GetComponentInChildren<T>();
        }

        public static void RenameChildren(Transform t)
        {
            for (var i = 0; i < t.childCount; i++)
            {
                t.GetChild(i).name = t.GetChild(i).name + " " + i.ToString();
            }
        }

        public static void SetMonobehavioursEnabled(GameObject gameObject, bool enabled)
        {
            foreach (MonoBehaviour c in gameObject.GetComponents<MonoBehaviour>())
            {
                c.enabled = enabled;
            }
        }

        public static T FindInChildrenAndParents<T>(this GameObject obj) where T : Component
        {
            T component;

            component = obj.GetComponentInParent<T>();
            if (component) return component;

            component = obj.GetComponentInChildren<T>();
            return component;
        }

        public static int Ceil(int a, int b)
        {
            return a > b ? b : a;
        }

        public static byte[] GetBytes(string str)
        {
            byte[] bytes = new byte[str.Length * sizeof(char)];
            System.Buffer.BlockCopy(str.ToCharArray(), 0, bytes, 0, bytes.Length);
            return bytes;
        }

        public static string GetString(byte[] bytes)
        {
            char[] chars = new char[bytes.Length / sizeof(char)];
            System.Buffer.BlockCopy(bytes, 0, chars, 0, bytes.Length);
            return new string(chars);
        }

        public static void Copy(Rigidbody2D source, Rigidbody2D target)
        {
            target.transform.position = source.transform.position;
            target.transform.rotation = source.transform.rotation;
            target.velocity = source.velocity;
            target.angularVelocity = source.angularVelocity;
        }

        #region math
        public static float DistancePointToLineSegment(Vector3 position, Vector3 linePointA, Vector3 linePointB)
        {
            float l2 = (linePointA - linePointB).sqrMagnitude;
            if (l2 == 0f) return (position - linePointA).magnitude;
            float t = Vector3.Dot(position - linePointA, linePointB - linePointA) / l2;
            if (t < 0f) return (position - linePointA).magnitude;
            else if (t > 1f) return (position - linePointB).magnitude;
            Vector3 projection = linePointA + t * (linePointB - linePointA);
            return (position - projection).magnitude;
        }
        #endregion
    }

}