using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class Game : MonoBehaviour
{
    private core.Core core;

    void OnEnable()
    {
        if (this.core == null)
        {
            this.core = new core.Core("");
        }
    }

    void Destroy()
    {
        this.core.Dispose();
        this.core = null;
    }
}
